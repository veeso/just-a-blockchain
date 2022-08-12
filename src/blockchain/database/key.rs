//! # key
//!
//! This module implements the key for leveldb

use crate::bridge::leveldb::Key;

/// The key used for the database (u64)
pub struct BlockKey(u64);

impl From<u64> for BlockKey {
    fn from(index: u64) -> Self {
        Self(index)
    }
}

impl Key for BlockKey {
    fn from_u8(key: &[u8]) -> Self {
        assert!(key.len() == 8);

        Self(
            (key[0] as u64) << 56
                | (key[1] as u64) << 48
                | (key[2] as u64) << 40
                | (key[3] as u64) << 32
                | (key[4] as u64) << 24
                | (key[5] as u64) << 16
                | (key[6] as u64) << 8
                | (key[7] as u64),
        )
    }

    fn as_slice<T, F: Fn(&[u8]) -> T>(&self, f: F) -> T {
        let mut dst = [0u8, 0, 0, 0, 0, 0, 0, 0];
        let value = self.0;
        dst[0] = (value >> 56) as u8;
        dst[1] = (value >> 48) as u8;
        dst[2] = (value >> 40) as u8;
        dst[3] = (value >> 32) as u8;
        dst[4] = (value >> 24) as u8;
        dst[5] = (value >> 16) as u8;
        dst[6] = (value >> 8) as u8;
        dst[7] = value as u8;
        f(&dst)
    }
}

#[cfg(test)]
mod test {

    use super::*;

    use pretty_assertions::assert_eq;

    #[test]
    fn should_be_able_to_use_u64_as_key() {
        let key = BlockKey::from(0xcafebabedeadbeef);
        assert_eq!(key.0, 0xcafebabedeadbeef);
        key.as_slice(|x| assert_eq!(x, &[0xca, 0xfe, 0xba, 0xbe, 0xde, 0xad, 0xbe, 0xef]));
        assert_eq!(
            BlockKey::from_u8(&[0xca, 0xfe, 0xba, 0xbe, 0xde, 0xad, 0xbe, 0xef]).0,
            0xcafebabedeadbeef
        );
    }
}
