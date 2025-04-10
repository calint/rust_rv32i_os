use core::ops::Deref;

#[derive(Clone, Copy)]
pub struct FixedSizeString<const N: usize> {
    data: [u8; N],
    len: usize,
}

impl<const N: usize> FixedSizeString<N> {
    pub const fn new() -> Self {
        Self {
            data: [0u8; N],
            len: 0,
        }
    }

    pub fn from(src: &[u8]) -> Self {
        Self::from_parts(&[src])
    }

    /// Will not write more than the length.
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

    /// Will not write more than the length. Silently returns self.
    pub fn append(&mut self, s: &[u8]) -> &Self {
        let cpy_len = s.len().min(N - self.len);
        self.data[self.len..self.len + cpy_len].copy_from_slice(&s[..cpy_len]);
        self.len += cpy_len;
        self
    }
}

impl<const N: usize> Default for FixedSizeString<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> Deref for FixedSizeString<N> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.data[..self.len]
    }
}

impl<const N: usize> PartialEq<&[u8]> for FixedSizeString<N> {
    fn eq(&self, other: &&[u8]) -> bool {
        &**self == *other
    }
}

impl<const N: usize> PartialEq<Self> for FixedSizeString<N> {
    fn eq(&self, other: &Self) -> bool {
        **self == **other
    }
}

impl<const N: usize> Eq for FixedSizeString<N> {}
