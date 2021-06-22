use core::str::FromStr;

use crate::heapless::String;
use crate::uluru::LRUCache;

pub trait History<const COMMAND_LEN: usize> {
    fn reset(&mut self);
    fn push(&mut self, command: &str) -> Result<(), ()>;
    fn go_back(&mut self) -> Option<String<COMMAND_LEN>>;
    fn go_forward(&mut self) -> Option<String<COMMAND_LEN>>;
}

pub struct NoHistory;

impl<const COMMAND_LEN: usize> History<COMMAND_LEN> for NoHistory {
    fn reset(&mut self) {}

    fn push(&mut self, _command: &str) -> Result<(), ()> {
        Ok(())
    }

    fn go_back(&mut self) -> Option<String<COMMAND_LEN>> {
        None
    }

    fn go_forward(&mut self) -> Option<String<COMMAND_LEN>> {
        None
    }
}

#[derive(Default)]
pub struct LRUHistory<const COMMAND_LEN: usize, const HISTORY_LEN: usize> {
    history: LRUCache<String<COMMAND_LEN>, HISTORY_LEN>,
    cursor: usize,
}

impl<const COMMAND_LEN: usize, const HISTORY_LEN: usize> History<COMMAND_LEN>
    for LRUHistory<COMMAND_LEN, HISTORY_LEN>
{
    fn reset(&mut self) {
        self.cursor = 0;
        self.history = LRUCache::default();
    }

    fn push(&mut self, command: &str) -> Result<(), ()> {
        if command.len() > 0 && HISTORY_LEN > 0 {
            if self.history.find(|item| item.as_str() == command).is_none() {
                let history_entry = String::from_str(command)?;
                self.history.insert(history_entry);
            }
        }
        self.cursor = 0;
        Ok(())
    }

    fn go_back(&mut self) -> Option<String<COMMAND_LEN>> {
        if self.history.len() == 0 || self.cursor == self.history.len() {
            None
        } else {
            let cursor = self.cursor;
            self.cursor += 1;
            self.history.get(cursor).cloned()
        }
    }

    fn go_forward(&mut self) -> Option<String<COMMAND_LEN>> {
        if self.cursor == 0 || self.history.len() == 0 {
            None
        } else {
            self.cursor -= 1;
            let cursor = self.cursor;
            self.history.get(cursor).cloned()
        }
    }
}
