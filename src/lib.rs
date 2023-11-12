#![no_std]
#![deny(unsafe_code)]

extern crate embedded_io as io;
extern crate heapless;
extern crate nb;
extern crate uluru;

use core::{fmt, marker::PhantomData, str::Utf8Error};
use io::{Error as IoError, ErrorKind, ErrorType, Read, Write};

pub mod autocomplete;
pub mod control;
pub mod history;

mod shell;

pub use shell::*;

pub enum ShellError {
    ReadError(ErrorKind),
    WriteError(ErrorKind),
    FormatError(fmt::Error),
    BadInputError(Utf8Error),
    WouldBlock,
    HistoryError,
}

impl fmt::Display for ShellError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ReadError(err) => write!(f, "Read error: {err:?}"),
            Self::WriteError(err) => write!(f, "Write error: {err:?}"),
            Self::FormatError(err) => write!(f, "Format error: {err}"),
            Self::BadInputError(err) => write!(f, "I/O error: {err}"),
            Self::WouldBlock => write!(f, "I/O transaction would block"),
            Self::HistoryError => write!(f, "Shell history error"),
        }
    }
}

impl fmt::Debug for ShellError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl From<fmt::Error> for ShellError {
    fn from(err: fmt::Error) -> Self {
        ShellError::FormatError(err)
    }
}

impl From<Utf8Error> for ShellError {
    fn from(err: Utf8Error) -> Self {
        ShellError::BadInputError(err)
    }
}

pub enum SpinError<E> {
    ShellError(ShellError),
    EnvironmentError(E),
}

impl<E> From<ShellError> for SpinError<E> {
    fn from(err: ShellError) -> Self {
        SpinError::ShellError(err)
    }
}

impl<E> From<fmt::Error> for SpinError<E> {
    fn from(err: fmt::Error) -> Self {
        SpinError::ShellError(err.into())
    }
}

impl<E> From<Utf8Error> for SpinError<E> {
    fn from(err: Utf8Error) -> Self {
        SpinError::ShellError(err.into())
    }
}

impl IoError for ShellError {
    fn kind(&self) -> ErrorKind {
        match self {
            Self::ReadError(err) => *err,
            Self::WriteError(err) => *err,
            _ => ErrorKind::Other,
        }
    }
}

pub enum Input<'a> {
    Control(u8),
    Command((&'a str, &'a str)),
}

pub trait Environment<S, A, H, E, const CMD_LEN: usize>
where
    S: Read + Write,
    A: autocomplete::Autocomplete<CMD_LEN>,
    H: history::History<CMD_LEN>,
{
    fn command(
        &mut self,
        shell: &mut UShell<S, A, H, CMD_LEN>,
        cmd: &str,
        args: &str,
    ) -> SpinResult<E>;

    fn control(&mut self, shell: &mut UShell<S, A, H, CMD_LEN>, code: u8) -> SpinResult<E>;
}

pub struct Serial<TX: Write, RX: Read> {
    w: PhantomData<u8>,
    tx: TX,
    rx: RX,
}

impl<TX: Write, RX: Read> ErrorType for Serial<TX, RX> {
    type Error = ShellError;
}

impl<TX: Write, RX: Read> Serial<TX, RX> {
    pub fn from_parts(tx: TX, rx: RX) -> Self {
        Self {
            tx,
            rx,
            w: PhantomData,
        }
    }

    pub fn tx(&mut self) -> &mut TX {
        &mut self.tx
    }

    pub fn rx(&mut self) -> &mut RX {
        &mut self.rx
    }

    pub fn split(self) -> (TX, RX) {
        (self.tx, self.rx)
    }
}

impl<TX: Write, RX: Read> Write for Serial<TX, RX> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        self.tx.write(buf).map_err(|err| Self::Error::WriteError(err.kind()))
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        self.tx.flush().map_err(|err| Self::Error::WriteError(err.kind()))
    }
}

impl<TX: Write, RX: Read> Read for Serial<TX, RX> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.rx.read(buf).map_err(|err| Self::Error::ReadError(err.kind()))
    }
}
