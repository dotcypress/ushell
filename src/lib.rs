#![no_std]
#![deny(unsafe_code)]

extern crate embedded_hal as hal;
extern crate heapless;
extern crate nb;
extern crate uluru;

use core::fmt;
use core::{marker::PhantomData, str::Utf8Error};
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
    WouldBlock,
    HistoryError,
}

pub enum Input<'a> {
    Control(u8),
    Command((&'a str, &'a str)),
}

pub struct Serial<W, TX: Write<W>, RX: Read<W>> {
    w: PhantomData<W>,
    tx: TX,
    rx: RX,
}

impl<W, TX: Write<W>, RX: Read<W>> Serial<W, TX, RX> {
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

impl<W, TX: Write<W>, RX: Read<W>> Write<W> for Serial<W, TX, RX> {
    type Error = TX::Error;

    fn write(&mut self, word: W) -> nb::Result<(), Self::Error> {
        self.tx.write(word)
    }

    fn flush(&mut self) -> nb::Result<(), Self::Error> {
        self.tx.flush()
    }
}

impl<W, TX: Write<W>, RX: Read<W>> Read<W> for Serial<W, TX, RX> {
    type Error = RX::Error;

    fn read(&mut self) -> nb::Result<W, Self::Error> {
        self.rx.read()
    }
}
