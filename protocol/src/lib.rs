#[macro_use(matches)]
extern crate matches;

const MAX_KEY_LEN: usize = 1 << 8;
const MAX_VALUE_LEN: usize = 1 << 12;
const MIN_KEY: &'static [u8] = &[0x00; MAX_KEY_LEN];
const MAX_KEY: &'static [u8] = &[0xff; MAX_KEY_LEN];

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

pub mod request;
pub mod response;
pub mod util;
