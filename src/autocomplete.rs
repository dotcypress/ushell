use core::str::FromStr;

use crate::heapless::String;

pub trait Autocomplete<const ITEM_LEN: usize> {
    fn suggest(&self, prefix: &str) -> Option<String<ITEM_LEN>>;
}

pub struct NoAutocomplete;

impl<const ITEM_LEN: usize> Autocomplete<ITEM_LEN> for NoAutocomplete {
    fn suggest(&self, _prefix: &str) -> Option<String<ITEM_LEN>> {
        None
    }
}

pub struct StaticAutocomplete<const N: usize>(pub [&'static str; N]);

impl<const ITEM_LEN: usize, const N: usize> Autocomplete<ITEM_LEN> for StaticAutocomplete<N> {
    fn suggest(&self, prefix: &str) -> Option<String<ITEM_LEN>> {
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
