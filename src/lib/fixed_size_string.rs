//
// reviewed: 2025-04-21
//
use core::ops::Deref;

#[derive(Clone, Copy)]
pub struct FixedSizeString<const SIZE: usize> {
    data: [u8; SIZE],
    len: usize,
}

impl<const SIZE: usize> FixedSizeString<SIZE> {
    pub const fn new() -> Self {
        Self {
            data: [0_u8; SIZE],
            len: 0,
        }
    }

    /// Will not write more than the allocated length.
    /// Silently returns self.
    pub fn from(source: &[u8]) -> Self {
        Self::from_parts(&[source])
    }

    /// Will not write more than the allocated length.
    /// Silently returns self.
    pub fn from_parts(parts: &[&[u8]]) -> Self {
        let mut s = Self::new();
        for &part in parts {
            s.append(part);
        }
        s
    }

    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Will not write more than the allocated length.
    /// Silently returns self.
    pub fn append(&mut self, source: &[u8]) -> &Self {
        let cpy_len = source.len().min(SIZE - self.len);
        self.data[self.len..self.len + cpy_len].copy_from_slice(&source[..cpy_len]);
        self.len += cpy_len;
        self
    }
}

impl<const SIZE: usize> Default for FixedSizeString<SIZE> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const SIZE: usize> Deref for FixedSizeString<SIZE> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.data[..self.len]
    }
}

impl<const SIZE: usize> PartialEq<&[u8]> for FixedSizeString<SIZE> {
    fn eq(&self, other: &&[u8]) -> bool {
        **self == **other
    }
}

impl<const SIZE: usize> PartialEq<Self> for FixedSizeString<SIZE> {
    fn eq(&self, other: &Self) -> bool {
        **self == **other
    }
}

impl<const N: usize> Eq for FixedSizeString<N> {}
