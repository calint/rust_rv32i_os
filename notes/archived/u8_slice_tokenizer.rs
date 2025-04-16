pub struct U8SliceTokenizer<'a> {
    slice: &'a [u8],
    index: usize,
    delimiter: fn(u8) -> bool,
}

impl<'a> U8SliceTokenizer<'a> {
    pub fn new(slice: &'a [u8], delimiter: fn(u8) -> bool) -> Self {
        U8SliceTokenizer {
            slice,
            index: 0,
            delimiter,
        }
    }

    pub fn rest(&self) -> &[u8] {
        &self.slice[self.index..self.slice.len()]
    }

    pub fn next(&mut self) -> Option<&[u8]> {
        // todo this can be done with only 2 loops by skipping the leading delimiters before creation of iterator
        // skip leading delimiters and find the start of the next chunk
        while self.index < self.slice.len() && (self.delimiter)(self.slice[self.index]) {
            self.index += 1;
        }

        if self.index >= self.slice.len() {
            return None;
        }

        // find the end of the chunk
        let start = self.index;
        while self.index < self.slice.len() && !(self.delimiter)(self.slice[self.index]) {
            self.index += 1;
        }

        // if the chunk is delimiters only, return None
        if start == self.index {
            return None;
        }

        let end = self.index;

        // skip trailing delimiters
        while self.index < self.slice.len() && (self.delimiter)(self.slice[self.index]) {
            self.index += 1;
        }

        Some(&self.slice[start..end])
    }
}
