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
pub enum InputKind<'a> {
    Control(u8),
    Command(&'a str),
}

pub struct Input<'a> {
    pub raw: u8,
    pub input: Option<InputKind<'a>>,
}

impl<'a> Input<'a> {
    fn raw(raw: u8) -> Self {
        Input { raw, input: None }
    }

    fn control(raw: u8) -> Self {
        Input {
            raw,
            input: Some(InputKind::Control(raw)),
        }
    }

    fn command(raw: u8, line: &'a str) -> Self {
        Input {
            raw,
            input: Some(InputKind::Command(line)),
        }
    }
}
