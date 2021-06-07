use core::{fmt::Write, str::from_utf8};
use hal::serial;
use nb::block;

use crate::autocomplete::Autocomplete;
use crate::history::History;
use crate::*;

pub struct UShell<S, A, H, const COMMAND_LEN: usize> {
    serial: S,
    autocomplete: A,
    history: H,
    cmd_buf: [u8; COMMAND_LEN],
    cmd_len: usize,
    cursor: usize,
    bypass: bool,
    control: bool,
    escape: bool,
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
            cmd_buf: [0; COMMAND_LEN],
            cmd_len: 0,
            cursor: 0,
            bypass: false,
            control: false,
            escape: false,
        }
    }

    pub fn bypass(&mut self, bypass_on: bool) {
        self.bypass = bypass_on;
    }

    pub fn reset(&mut self) {
        self.history.reset();
        self.control = false;
        self.escape = false;
        self.cursor = 0;
        self.cmd_len = 0;
    }

    pub fn poll(&mut self) -> PollResult<S> {
        const ANSI_ESCAPE: u8 = b'[';

        match self.serial.read() {
            Ok(byte) => {
                if self.bypass {
                    return Ok(Some(Input::Raw(byte)));
                }

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
                            _ => return Ok(Some(Input::Control(control_byte))),
                        }
                    }
                    _ if self.escape => {
                        self.escape = false;
                        self.control = false;
                    }
                    control::TAB => self.autocomplete()?,
                    control::DEL | control::BS => self.delete_at_cursor()?,
                    control::CR => {
                        let cmd = from_utf8(&self.cmd_buf[..self.cmd_len])
                            .map_err(ShellError::BadInputError)?;
                        self.history.push(cmd).map_err(ShellError::HistoryError)?;
                        self.cmd_len = 0;
                        self.cursor = 0;
                        return Ok(Some(Input::Command(byte, cmd)));
                    }
                    _ => self.write_at_cursor(byte)?,
                };
                Ok(Some(Input::Raw(byte)))
            }
            Err(nb::Error::WouldBlock) => Ok(None),
            Err(nb::Error::Other(err)) => Err(ShellError::ReadError(err)),
        }
    }

    pub fn clear(&mut self) -> ShellResult<S> {
        self.cursor = 0;
        self.cmd_len = 0;
        self.write_str("\x1b[H\x1b[2J")
            .map_err(ShellError::FormatError)
    }

    pub fn bell(&mut self) -> ShellResult<S> {
        block!(self.serial.write(control::BELL)).map_err(ShellError::WriteError)
    }

    pub fn push_history(&mut self, cmd: &str) -> ShellResult<S> {
        self.history.push(cmd).map_err(ShellError::HistoryError)
    }

    fn write_at_cursor(&mut self, byte: u8) -> ShellResult<S> {
        if self.cursor == self.cmd_buf.len() {
            return self.bell();
        } else if self.cursor < self.cmd_len {
            block!(self.serial.write(byte)).map_err(ShellError::WriteError)?;

            self.cmd_buf
                .copy_within(self.cursor..self.cmd_len, self.cursor + 1);
            self.cmd_buf[self.cursor] = byte;
            self.cursor += 1;
            self.cmd_len += 1;

            self.write_str("\x1b[s\x1b[K")
                .map_err(ShellError::FormatError)?;
            for b in &self.cmd_buf[self.cursor..self.cmd_len] {
                block!(self.serial.write(*b)).map_err(ShellError::WriteError)?;
            }
            self.write_str("\x1b[u").map_err(ShellError::FormatError)
        } else {
            self.cmd_buf[self.cursor] = byte;
            self.cursor += 1;
            self.cmd_len += 1;
            block!(self.serial.write(byte)).map_err(ShellError::WriteError)
        }
    }

    fn delete_at_cursor(&mut self) -> ShellResult<S> {
        if self.cursor == 0 {
            self.bell()?;
            return Ok(());
        } else if self.cursor < self.cmd_len {
            self.cmd_buf
                .copy_within(self.cursor..self.cmd_len, self.cursor - 1);
            self.cursor -= 1;
            self.cmd_len -= 1;
            self.write_str("\x1b[D\x1b[s\x1b[K")
                .map_err(ShellError::FormatError)?;
            for b in &self.cmd_buf[self.cursor..self.cmd_len] {
                block!(self.serial.write(*b)).map_err(ShellError::WriteError)?;
            }
            self.write_str("\x1b[u").map_err(ShellError::FormatError)
        } else {
            self.cursor -= 1;
            self.cmd_len -= 1;
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
        if self.cursor < self.cmd_len {
            self.cursor += 1;
            self.write_str("\x1b[C").map_err(ShellError::FormatError)
        } else {
            self.bell()
        }
    }

    fn dpad_up(&mut self) -> ShellResult<S> {
        if self.cursor != self.cmd_len {
            return self.bell();
        }
        match self.history.go_back() {
            None => self.bell(),
            Some(cmd) => self.replace_cmd_buf(cmd.as_str()),
        }
    }

    fn dpad_down(&mut self) -> ShellResult<S> {
        if self.cursor != self.cmd_len {
            return self.bell();
        }
        match self.history.go_forward() {
            None => self.bell(),
            Some(cmd) => self.replace_cmd_buf(cmd.as_str()),
        }
    }

    fn autocomplete(&mut self) -> ShellResult<S> {
        let prefix = from_utf8(&self.cmd_buf[..self.cursor]).map_err(ShellError::BadInputError)?;
        match self.autocomplete.suggest(prefix) {
            None => self.bell(),
            Some(suffix) => {
                let bytes = suffix.as_bytes();
                self.cmd_buf[self.cursor..(self.cursor + bytes.len())].copy_from_slice(bytes);
                self.cursor += bytes.len();
                self.cmd_len = self.cursor;
                write!(self, "\x1b[K{}", suffix.as_str()).map_err(ShellError::FormatError)
            }
        }
    }

    fn replace_cmd_buf(&mut self, cmd: &str) -> ShellResult<S> {
        let cursor = self.cursor;
        if cursor > 0 {
            write!(self, "\x1b[{}D", cursor).map_err(ShellError::FormatError)?;
        }

        let bytes = cmd.as_bytes();
        self.cmd_len = bytes.len();
        self.cursor = bytes.len();
        self.cmd_buf[..bytes.len()].copy_from_slice(bytes);
        write!(self, "\x1b[K{}", cmd).map_err(ShellError::FormatError)
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
