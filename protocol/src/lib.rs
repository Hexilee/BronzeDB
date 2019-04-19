#[cfg(test)]
extern crate matches;

#[cfg(test)]
extern crate speculate;

const MAX_KEY_LEN: usize = 1 << 8;
const MAX_VALUE_LEN: usize = 1 << 12;
const MIN_KEY: &'static [u8] = b"";
const MAX_KEY: &'static [u8] = &[0xff; MAX_KEY_LEN];

pub mod ext;
pub mod request;
pub mod response;
