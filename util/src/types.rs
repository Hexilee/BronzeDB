use libc::memcmp;
use std::cmp::{Eq, Ordering, PartialEq, PartialOrd};
use std::hash::{Hash, Hasher};
use std::mem::transmute;
use std::ops::Deref;
use std::os::raw::c_void;

pub type Value = Vec<u8>;
pub type Entry = (Key, Value);
pub type EntryRef<'a> = (&'a Key, &'a Value);

#[derive(Debug, Clone, Default)]
pub struct Key {
    data: Vec<u8>,
}

pub struct RawKey {
    data: [u8],
}

impl Deref for RawKey {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl Deref for Key {
    type Target = Vec<u8>;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl From<&[u8]> for &RawKey {
    fn from(data: &[u8]) -> Self {
        unsafe { transmute(data) }
    }
}

impl From<Vec<u8>> for Key {
    fn from(data: Vec<u8>) -> Self {
        Self { data }
    }
}

impl PartialEq for RawKey {
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

impl PartialOrd for RawKey {
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

impl PartialEq for Key {
    fn eq(&self, other: &Self) -> bool {
        <RawKey as PartialEq>::eq(self.as_slice().into(), other.as_slice().into())
    }
}

impl PartialOrd for Key {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        <RawKey as PartialOrd>::partial_cmp(self.as_slice().into(), other.as_slice().into())
    }
}

impl Eq for Key {}

impl Hash for Key {
    fn hash<H: Hasher>(&self, state: &mut H) {
        <u8 as Hash>::hash_slice(self.as_slice(), state)
    }
}

#[cfg(test)]
mod tests {
    use std::cmp::Ordering::*;
    use std::cmp::PartialOrd;

    #[test]
    fn key_cmp() {
        for (former, latter, order) in vec![
            ("xixi", "haha", Greater),
            ("haa", "haha", Less),
            ("haha", "haha", Equal),
            ("hah", "haha", Less),
            ("hahah", "haha", Greater),
        ] {
            let former_key = former.to_owned().into_bytes();
            let latter_key = latter.to_owned().into_bytes();
            assert_eq!(order, former_key.partial_cmp(&latter_key).unwrap());
        }
    }
}
