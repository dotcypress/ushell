#![no_std]
#![deny(warnings)]
#![deny(unsafe_code)]

extern crate embedded_hal as hal;
extern crate heapless;
extern crate nb;
extern crate uluru;

use core::{fmt, str::Utf8Error};
use hal::serial::{Read, Write};

pub mod autocomplete;
pub mod control;
pub mod history;

mod shell;

pub use shell::*;

pub enum ShellError<S>
where
    S: Read<u8> + Write<u8>,
{
    ReadError(<S as Read<u8>>::Error),
    WriteError(<S as Write<u8>>::Error),
    FormatError(fmt::Error),
    BadInputError(Utf8Error),
    HistoryError(()),
}

pub type ShellResult<S> = Result<(), ShellError<S>>;
pub type PollResult<'a, S> = Result<Option<Input<'a>>, ShellError<S>>;
pub enum Input<'a> {
    Raw(u8),
    Control(u8),
    Command(u8, &'a str),
}
