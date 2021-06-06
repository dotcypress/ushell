use crate::heapless::String;

pub trait History<const COMMAND_LEN: usize> {
    fn reset(&mut self);
    fn push_top(&mut self, command: &str) -> Result<(), ()>;
    fn go_back(&mut self) -> Option<String<COMMAND_LEN>>;
    fn go_forward(&mut self) -> Option<String<COMMAND_LEN>>;
}

pub struct NoHistory;

impl<const COMMAND_LEN: usize> History<COMMAND_LEN> for NoHistory {
    fn reset(&mut self) {}

    fn push_top(&mut self, _command: &str) -> Result<(), ()> {
        Ok(())
    }

    fn go_back(&mut self) -> Option<String<COMMAND_LEN>> {
        None
    }

    fn go_forward(&mut self) -> Option<String<COMMAND_LEN>> {
        None
    }
}
