use core::{fmt::Write, str::from_utf8};

use crate::autocomplete::Autocomplete;
use crate::history::History;
use crate::*;

pub type ShellResult = Result<(), ShellError>;
pub type SpinResult<E> = Result<(), SpinError<E>>;
pub type PollResult<'a> = Result<Option<Input<'a>>, ShellError>;

pub struct UShell<S, A, H, const CMD_LEN: usize> {
    serial: S,
    autocomplete: A,
    history: H,
    editor_buf: [u8; CMD_LEN],
    editor_len: usize,
    cursor: usize,
    control: bool,
    escape: bool,
    autocomplete_on: bool,
    history_on: bool,
}

impl<S, A, H, const CMD_LEN: usize> UShell<S, A, H, CMD_LEN>
where
    S: io::Read + io::Write,
    A: Autocomplete<CMD_LEN>,
    H: History<CMD_LEN>,
{
    pub fn new(serial: S, autocomplete: A, history: H) -> Self {
        Self {
            serial,
            autocomplete,
            history,
            cursor: 0,
            editor_buf: [0; CMD_LEN],
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

    pub fn spin<E, ENV: Environment<S, A, H, E, CMD_LEN>>(
        &mut self,
        env: &mut ENV,
    ) -> SpinResult<E> {
        loop {
            match self.poll() {
                Err(ShellError::WouldBlock) => return Ok(()),
                Err(err) => return Err(SpinError::ShellError(err)),
                Ok(None) => continue,
                Ok(Some(Input::Control(code))) => env.control(self, code)?,
                Ok(Some(Input::Command((cmd, args)))) => {
                    let mut cmd_buf = [0; CMD_LEN];
                    cmd_buf[..cmd.len()].copy_from_slice(cmd.as_bytes());
                    let cmd = core::str::from_utf8(&cmd_buf[..cmd.len()])?;

                    let mut args_buf = [0; CMD_LEN];
                    args_buf[..args.len()].copy_from_slice(args.as_bytes());
                    let args = core::str::from_utf8(&args_buf[..args.len()])?;

                    env.command(self, cmd, args)?
                }
            };
        }
    }

    pub fn poll(&mut self) -> PollResult {
        const ANSI_ESCAPE: u8 = b'[';

        let mut byte = [0u8];
        match self.serial.read(&mut byte) {
            Ok(_read) => {
                match byte[0] {
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
                        let line = from_utf8(&self.editor_buf[..self.editor_len])?;
                        self.history
                            .push(line)
                            .map_err(|_| ShellError::HistoryError)?;
                        self.editor_len = 0;
                        self.cursor = 0;
                        return Ok(Some(Input::Command(
                            line.split_once(' ').unwrap_or((line, "")),
                        )));
                    }
                    _ => {
                        let ch = byte[0] as char;
                        if ch.is_ascii_control() {
                            return Ok(Some(Input::Control(byte[0])));
                        } else {
                            self.write_at_cursor(byte[0])?;
                        }
                    }
                };
                Ok(None)
            }
            Err(err) => Err(ShellError::ReadError(err.kind())),
        }
    }

    pub fn clear(&mut self) -> ShellResult {
        self.cursor = 0;
        self.editor_len = 0;
        self.write_str("\x1b[H\x1b[2J")?;
        Ok(())
    }

    pub fn bell(&mut self) -> ShellResult {
        self.serial
            .write(&[control::BELL])
            .map(|_| ())
            .map_err(|err| ShellError::WriteError(err.kind()))
    }

    pub fn push_history(&mut self, line: &str) -> ShellResult {
        self.history
            .push(line)
            .map_err(|_| ShellError::HistoryError)
    }

    fn write_at_cursor(&mut self, byte: u8) -> ShellResult {
        if self.cursor == self.editor_buf.len() {
            return self.bell();
        }

        if self.cursor < self.editor_len && self.editor_len < CMD_LEN {
            self.serial.write(&[byte]).map_err(|err| ShellError::WriteError(err.kind()))?;
            self.editor_buf
                .copy_within(self.cursor..self.editor_len, self.cursor + 1);
            self.editor_buf[self.cursor] = byte;
            self.cursor += 1;
            self.editor_len += 1;
            self.write_str("\x1b[s\x1b[K")?;
            self.serial
                .write(&self.editor_buf[self.cursor..self.editor_len])
                .map_err(|err| ShellError::WriteError(err.kind()))?;
            self.write_str("\x1b[u")?;
        } else if self.cursor < self.editor_len {
            self.serial.write(&[byte]).map_err(|err| ShellError::WriteError(err.kind()))?;
            self.editor_buf[self.cursor] = byte;
            self.cursor += 1;
        } else {
            self.serial.write(&[byte]).map_err(|err| ShellError::WriteError(err.kind()))?;
            self.editor_buf[self.cursor] = byte;
            self.cursor += 1;
            self.editor_len += 1;
        }
        Ok(())
    }

    fn delete_at_cursor(&mut self) -> ShellResult {
        if self.cursor == 0 {
            self.bell()?;
        } else if self.cursor < self.editor_len {
            self.editor_buf
                .copy_within(self.cursor..self.editor_len, self.cursor - 1);
            self.cursor -= 1;
            self.editor_len -= 1;
            self.write_str("\x1b[D\x1b[s\x1b[K")?;
            self.serial
                .write(&self.editor_buf[self.cursor..self.editor_len])
                .map_err(|err| ShellError::WriteError(err.kind()))?;
            self.write_str("\x1b[u")?;
        } else {
            self.cursor -= 1;
            self.editor_len -= 1;
            self.write_str("\x08 \x08")?;
        }
        Ok(())
    }

    fn dpad_left(&mut self) -> ShellResult {
        if self.cursor > 0 {
            self.cursor -= 1;
            self.write_str("\x1b[D")?;
        } else {
            self.bell()?;
        }
        Ok(())
    }

    fn dpad_right(&mut self) -> ShellResult {
        if self.cursor < self.editor_len {
            self.cursor += 1;
            self.write_str("\x1b[C")?;
        } else {
            self.bell()?;
        }
        Ok(())
    }

    fn dpad_up(&mut self) -> ShellResult {
        if self.cursor != self.editor_len || !self.history_on {
            return self.bell();
        }
        match self.history.go_back() {
            None => self.bell(),
            Some(line) => self.replace_editor_buf(line.as_str()),
        }
    }

    fn dpad_down(&mut self) -> ShellResult {
        if self.cursor != self.editor_len || !self.history_on {
            return self.bell();
        }
        match self.history.go_forward() {
            None => self.bell(),
            Some(line) => self.replace_editor_buf(line.as_str()),
        }
    }

    fn suggest(&mut self) -> ShellResult {
        let prefix = from_utf8(&self.editor_buf[..self.cursor])?;
        match self.autocomplete.suggest(prefix) {
            None => self.bell()?,
            Some(suffix) => {
                let bytes = suffix.as_bytes();
                self.editor_buf[self.cursor..(self.cursor + bytes.len())].copy_from_slice(bytes);
                self.cursor += bytes.len();
                self.editor_len = self.cursor;
                write!(self, "\x1b[K{}", suffix.as_str())?;
            }
        }
        Ok(())
    }

    fn replace_editor_buf(&mut self, line: &str) -> ShellResult {
        let cursor = self.cursor;
        if cursor > 0 {
            write!(self, "\x1b[{}D", cursor)?;
        }

        let bytes = line.as_bytes();
        self.editor_len = bytes.len();
        self.cursor = bytes.len();
        self.editor_buf[..bytes.len()].copy_from_slice(bytes);
        write!(self, "\x1b[K{}", line)?;
        Ok(())
    }
}

impl<S, A, H, const CMD_LEN: usize> fmt::Write for UShell<S, A, H, CMD_LEN>
where
    S: io::Read + io::Write,
    A: Autocomplete<CMD_LEN>,
    H: History<CMD_LEN>,
{
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.serial
            .write(s.as_bytes())
            .map(|_| ())
            .map_err(|_| fmt::Error::default())
    }
}
