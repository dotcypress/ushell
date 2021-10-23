use core::str::FromStr;

use crate::heapless::String;

pub trait Autocomplete<const CMD_LEN: usize> {
    fn suggest(&self, prefix: &str) -> Option<String<CMD_LEN>>;
}

pub struct NoAutocomplete;

impl<const CMD_LEN: usize> Autocomplete<CMD_LEN> for NoAutocomplete {
    fn suggest(&self, _prefix: &str) -> Option<String<CMD_LEN>> {
        None
    }
}

pub struct FnAutocomplete<const CMD_LEN: usize>(fn(&str) -> Option<String<CMD_LEN>>);

impl<const CMD_LEN: usize> Autocomplete<CMD_LEN> for FnAutocomplete<CMD_LEN> {
    fn suggest(&self, prefix: &str) -> Option<String<CMD_LEN>> {
        self.0(prefix)
    }
}

pub struct StaticAutocomplete<const N: usize>(pub [&'static str; N]);

impl<const CMD_LEN: usize, const N: usize> Autocomplete<CMD_LEN> for StaticAutocomplete<N> {
    fn suggest(&self, prefix: &str) -> Option<String<CMD_LEN>> {
        if prefix.len() == 0 {
            return None;
        }
        for item in self.0.iter() {
            if item.starts_with(prefix) {
                let (_, suffix) = item.split_at(prefix.len());
                return String::from_str(suffix).ok();
            }
        }
        None
    }
}
