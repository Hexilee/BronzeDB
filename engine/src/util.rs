use libc::memcmp;
use std::cmp::{Ordering, PartialEq, PartialOrd};
use std::ops::Deref;
use std::os::raw::c_void;

pub type Value<'a> = &'a [u8];
pub type Entry<'a> = (Key<'a>, Value<'a>);

pub struct Key<'a> {
    data: &'a [u8],
}

impl<'a> Deref for Key<'a> {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        self.data
    }
}

impl<'a> From<&'a [u8]> for Key<'a> {
    fn from(data: &'a [u8]) -> Self {
        Self { data }
    }
}

impl<'a> PartialEq for Key<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.len() == other.len()
            && unsafe {
                0 == memcmp(
                    &self.data[0] as *const u8 as *const c_void,
                    &other.data[0] as *const u8 as *const c_void,
                    self.len(),
                )
            }
    }
}

impl<'a> PartialOrd for Key<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match unsafe {
            memcmp(
                &self.data[0] as *const u8 as *const c_void,
                &other.data[0] as *const u8 as *const c_void,
                self.len(),
            )
        } {
            x if x < 0 => Some(Ordering::Less),
            x if x > 0 => Some(Ordering::Greater),
            _ => {
                if self.len() == other.len() {
                    Some(Ordering::Equal)
                } else if self.len() < other.len() {
                    Some(Ordering::Less)
                } else {
                    Some(Ordering::Greater)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Key;
    use std::cmp::Ordering::*;
    use std::cmp::PartialOrd;
    #[test]
    fn key_cmp() {
        for (former, latter, order) in vec![
            (&b"xixi"[..], &b"haha"[..], Greater),
            (&b"haa"[..], &b"haha"[..], Less),
            (&b"haha"[..], &b"haha"[..], Equal),
            (&b"hah"[..], &b"haha"[..], Less),
            (&b"hahah"[..], &b"haha"[..], Greater),
        ] {
            let former_key: Key = former.into();
            let latter_key = latter.into();
            assert_eq!(order, former_key.partial_cmp(&latter_key).unwrap());
        }
    }
}
