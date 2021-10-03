use core::{fmt::Write, str::from_utf8};
use hal::serial;
use nb::block;

use crate::autocomplete::Autocomplete;
use crate::history::History;
use crate::*;

pub type ShellResult<S> = Result<(), ShellError<S>>;
pub type SpinResult<S, E> = Result<(), SpinError<S, E>>;
pub type PollResult<'a, S> = Result<Option<Input<'a>>, ShellError<S>>;

pub struct UShell<S, A, H, const COMMAND_LEN: usize> {
    serial: S,
    autocomplete: A,
    history: H,
    editor_buf: [u8; COMMAND_LEN],
    editor_len: usize,
    cursor: usize,
    control: bool,
    escape: bool,
    autocomplete_on: bool,
    history_on: bool,
}

impl<S, A, H, const COMMAND_LEN: usize> UShell<S, A, H, COMMAND_LEN>
where
    S: serial::Read<u8> + serial::Write<u8>,
    A: Autocomplete<COMMAND_LEN>,
    H: History<COMMAND_LEN>,
{
    pub fn new(serial: S, autocomplete: A, history: H) -> Self {
        Self {
            serial,
            autocomplete,
            history,
            cursor: 0,
            editor_buf: [0; COMMAND_LEN],
            editor_len: 0,
            autocomplete_on: true,
            history_on: true,
            control: false,
            escape: false,
        }
    }

    pub fn autocomplete(&mut self, autocomplete_on: bool) {
        self.autocomplete_on = autocomplete_on;
    }

    pub fn history(&mut self, history_on: bool) {
        self.history_on = history_on;
    }

    pub fn get_autocomplete_mut(&mut self) -> &mut A {
        &mut self.autocomplete
    }

    pub fn get_history_mut(&mut self) -> &mut H {
        &mut self.history
    }

    pub fn get_serial_mut(&mut self) -> &mut S {
        &mut self.serial
    }

    pub fn reset(&mut self) {
        self.control = false;
        self.escape = false;
        self.cursor = 0;
        self.editor_len = 0;
    }

    pub fn spin<E, ENV: Environment<S, A, H, E, COMMAND_LEN>>(
        &mut self,
        env: &mut ENV,
    ) -> SpinResult<S, E> {
        loop {
            match self.poll() {
                Err(ShellError::WouldBlock) => return Ok(()),
                Err(err) => return Err(SpinError::ShellError(err)),
                Ok(None) => continue,
                Ok(Some(Input::Control(code))) => env.control(self, code)?,
                Ok(Some(Input::Command((cmd, args)))) => {
                    let mut cmd_buf = [0; COMMAND_LEN];
                    cmd_buf[..cmd.len()].copy_from_slice(cmd.as_bytes());
                    let cmd = core::str::from_utf8(&cmd_buf[..cmd.len()])
                        .map_err(ShellError::BadInputError)
                        .map_err(SpinError::ShellError)?;

                    let mut args_buf = [0; COMMAND_LEN];
                    args_buf[..args.len()].copy_from_slice(args.as_bytes());
                    let args = core::str::from_utf8(&args_buf[..args.len()])
                        .map_err(ShellError::BadInputError)
                        .map_err(SpinError::ShellError)?;

                    env.command(self, cmd, args)?
                }
            };
        }
    }

    pub fn poll(&mut self) -> PollResult<S> {
        const ANSI_ESCAPE: u8 = b'[';

        match self.serial.read() {
            Ok(byte) => {
                match byte {
                    ANSI_ESCAPE if self.escape => {
                        self.control = true;
                    }
                    control::ESC => {
                        self.escape = true;
                    }
                    control_byte if self.control => {
                        self.escape = false;
                        self.control = false;

                        const UP: u8 = 0x41;
                        const DOWN: u8 = 0x42;
                        const RIGHT: u8 = 0x43;
                        const LEFT: u8 = 0x44;
                        match control_byte {
                            LEFT => self.dpad_left()?,
                            RIGHT => self.dpad_right()?,
                            UP => self.dpad_up()?,
                            DOWN => self.dpad_down()?,
                            _ => {}
                        }
                    }
                    _ if self.escape => {
                        self.escape = false;
                        self.control = false;
                    }
                    control::TAB => {
                        if self.autocomplete_on {
                            self.suggest()?
                        } else {
                            self.bell()?
                        }
                    }
                    control::DEL | control::BS => self.delete_at_cursor()?,
                    control::CR => {
                        let line = from_utf8(&self.editor_buf[..self.editor_len])
                            .map_err(ShellError::BadInputError)?;
                        self.history
                            .push(line)
                            .map_err(|_| ShellError::HistoryError)?;
                        self.editor_len = 0;
                        self.cursor = 0;
                        return Ok(Some(Input::Command(
                            line.split_once(" ").unwrap_or((line, &"")),
                        )));
                    }
                    _ => {
                        let ch = byte as char;
                        if ch.is_ascii_control() {
                            return Ok(Some(Input::Control(byte)));
                        } else {
                            self.write_at_cursor(byte)?;
                        }
                    }
                };
                Ok(None)
            }
            Err(nb::Error::WouldBlock) => Err(ShellError::WouldBlock),
            Err(nb::Error::Other(err)) => Err(ShellError::ReadError(err)),
        }
    }

    pub fn clear(&mut self) -> ShellResult<S> {
        self.cursor = 0;
        self.editor_len = 0;
        self.write_str("\x1b[H\x1b[2J")
            .map_err(ShellError::FormatError)
    }

    pub fn bell(&mut self) -> ShellResult<S> {
        block!(self.serial.write(control::BELL)).map_err(ShellError::WriteError)
    }

    pub fn push_history(&mut self, line: &str) -> ShellResult<S> {
        self.history
            .push(line)
            .map_err(|_| ShellError::HistoryError)
    }

    fn write_at_cursor(&mut self, byte: u8) -> ShellResult<S> {
        if self.cursor == self.editor_buf.len() {
            return self.bell();
        } else if self.cursor < self.editor_len {
            block!(self.serial.write(byte)).map_err(ShellError::WriteError)?;

            self.editor_buf
                .copy_within(self.cursor..self.editor_len, self.cursor + 1);
            self.editor_buf[self.cursor] = byte;
            self.cursor += 1;
            self.editor_len += 1;

            self.write_str("\x1b[s\x1b[K")
                .map_err(ShellError::FormatError)?;
            for b in &self.editor_buf[self.cursor..self.editor_len] {
                block!(self.serial.write(*b)).map_err(ShellError::WriteError)?;
            }
            self.write_str("\x1b[u").map_err(ShellError::FormatError)
        } else {
            self.editor_buf[self.cursor] = byte;
            self.cursor += 1;
            self.editor_len += 1;
            block!(self.serial.write(byte)).map_err(ShellError::WriteError)
        }
    }

    fn delete_at_cursor(&mut self) -> ShellResult<S> {
        if self.cursor == 0 {
            self.bell()?;
            return Ok(());
        } else if self.cursor < self.editor_len {
            self.editor_buf
                .copy_within(self.cursor..self.editor_len, self.cursor - 1);
            self.cursor -= 1;
            self.editor_len -= 1;
            self.write_str("\x1b[D\x1b[s\x1b[K")
                .map_err(ShellError::FormatError)?;
            for b in &self.editor_buf[self.cursor..self.editor_len] {
                block!(self.serial.write(*b)).map_err(ShellError::WriteError)?;
            }
            self.write_str("\x1b[u").map_err(ShellError::FormatError)
        } else {
            self.cursor -= 1;
            self.editor_len -= 1;
            self.write_str("\x08 \x08").map_err(ShellError::FormatError)
        }
    }

    fn dpad_left(&mut self) -> ShellResult<S> {
        if self.cursor > 0 {
            self.cursor -= 1;
            self.write_str("\x1b[D").map_err(ShellError::FormatError)
        } else {
            self.bell()
        }
    }

    fn dpad_right(&mut self) -> ShellResult<S> {
        if self.cursor < self.editor_len {
            self.cursor += 1;
            self.write_str("\x1b[C").map_err(ShellError::FormatError)
        } else {
            self.bell()
        }
    }

    fn dpad_up(&mut self) -> ShellResult<S> {
        if self.cursor != self.editor_len || !self.history_on {
            return self.bell();
        }
        match self.history.go_back() {
            None => self.bell(),
            Some(line) => self.replace_editor_buf(line.as_str()),
        }
    }

    fn dpad_down(&mut self) -> ShellResult<S> {
        if self.cursor != self.editor_len || !self.history_on {
            return self.bell();
        }
        match self.history.go_forward() {
            None => self.bell(),
            Some(line) => self.replace_editor_buf(line.as_str()),
        }
    }

    fn suggest(&mut self) -> ShellResult<S> {
        let prefix =
            from_utf8(&self.editor_buf[..self.cursor]).map_err(ShellError::BadInputError)?;
        match self.autocomplete.suggest(prefix) {
            None => self.bell(),
            Some(suffix) => {
                let bytes = suffix.as_bytes();
                self.editor_buf[self.cursor..(self.cursor + bytes.len())].copy_from_slice(bytes);
                self.cursor += bytes.len();
                self.editor_len = self.cursor;
                write!(self, "\x1b[K{}", suffix.as_str()).map_err(ShellError::FormatError)
            }
        }
    }

    fn replace_editor_buf(&mut self, line: &str) -> ShellResult<S> {
        let cursor = self.cursor;
        if cursor > 0 {
            write!(self, "\x1b[{}D", cursor).map_err(ShellError::FormatError)?;
        }

        let bytes = line.as_bytes();
        self.editor_len = bytes.len();
        self.cursor = bytes.len();
        self.editor_buf[..bytes.len()].copy_from_slice(bytes);
        write!(self, "\x1b[K{}", line).map_err(ShellError::FormatError)
    }
}

impl<S, A, H, const COMMAND_LEN: usize> fmt::Write for UShell<S, A, H, COMMAND_LEN>
where
    S: serial::Read<u8> + serial::Write<u8>,
    A: Autocomplete<COMMAND_LEN>,
    H: History<COMMAND_LEN>,
{
    fn write_str(&mut self, s: &str) -> fmt::Result {
        s.as_bytes()
            .iter()
            .map(|c| block!(self.serial.write(*c)))
            .last();
        Ok(())
    }
}
