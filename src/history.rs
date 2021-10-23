use core::str::FromStr;

use crate::heapless::String;
use crate::uluru::LRUCache;

pub trait History<const CMD_LEN: usize> {
    fn reset(&mut self);
    fn push(&mut self, command: &str) -> Result<(), ()>;
    fn go_back(&mut self) -> Option<String<CMD_LEN>>;
    fn go_forward(&mut self) -> Option<String<CMD_LEN>>;
}

pub struct NoHistory;

impl<const CMD_LEN: usize> History<CMD_LEN> for NoHistory {
    fn reset(&mut self) {}

    fn push(&mut self, _command: &str) -> Result<(), ()> {
        Ok(())
    }

    fn go_back(&mut self) -> Option<String<CMD_LEN>> {
        None
    }

    fn go_forward(&mut self) -> Option<String<CMD_LEN>> {
        None
    }
}

#[derive(Default)]
pub struct LRUHistory<const CMD_LEN: usize, const CAP: usize> {
    history: LRUCache<String<CMD_LEN>, CAP>,
    cursor: usize,
}

impl<const CMD_LEN: usize, const CAP: usize> History<CMD_LEN> for LRUHistory<CMD_LEN, CAP> {
    fn reset(&mut self) {
        self.cursor = 0;
        self.history = LRUCache::default();
    }

    fn push(&mut self, command: &str) -> Result<(), ()> {
        if command.len() > 0 && CAP > 0 {
            if self.history.find(|item| item.as_str() == command).is_none() {
                let history_entry = String::from_str(command)?;
                self.history.insert(history_entry);
            }
        }
        self.cursor = 0;
        Ok(())
    }

    fn go_back(&mut self) -> Option<String<CMD_LEN>> {
        if self.history.len() == 0 || self.cursor == self.history.len() {
            None
        } else {
            let cursor = self.cursor;
            self.cursor += 1;
            self.history.get(cursor).cloned()
        }
    }

    fn go_forward(&mut self) -> Option<String<CMD_LEN>> {
        if self.cursor == 0 || self.history.len() == 0 {
            None
        } else {
            self.cursor -= 1;
            let cursor = self.cursor;
            self.history.get(cursor).cloned()
        }
    }
}
