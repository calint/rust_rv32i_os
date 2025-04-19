pub struct CursorBuffer<const SIZE: usize, T> {
    buffer: [T; SIZE],
    end: usize,
    cursor: usize,
}

pub type Result<T> = core::result::Result<T, CursorBufferError>;

pub enum CursorBufferError {
    BufferFull,
    CursorAtStart,
    CursorAtEnd,
}

impl<const SIZE: usize, T> CursorBuffer<SIZE, T>
where
    T: Default + Copy,
{
    pub fn new() -> Self {
        Self {
            buffer: [T::default(); SIZE],
            end: 0,
            cursor: 0,
        }
    }

    pub fn insert(&mut self, ch: T) -> Result<()> {
        if self.end == SIZE {
            return Err(CursorBufferError::BufferFull);
        }

        if self.cursor == self.end {
            self.buffer[self.cursor] = ch;
            self.cursor += 1;
            self.end += 1;
        } else {
            self.end += 1;
            self.buffer
                .copy_within(self.cursor..self.end - 1, self.cursor + 1);
            self.buffer[self.cursor] = ch;
            self.cursor += 1;
        }
        Ok(())
    }

    pub fn delete(&mut self) -> Result<()> {
        if self.cursor == self.end {
            return Err(CursorBufferError::CursorAtEnd);
        }

        self.buffer
            .copy_within(self.cursor + 1..self.end, self.cursor);
        self.end -= 1;

        Ok(())
    }

    pub fn backspace(&mut self) -> Result<()> {
        if self.cursor == 0 {
            return Err(CursorBufferError::CursorAtStart);
        }

        if self.cursor == self.end {
            self.end -= 1;
            self.cursor -= 1;
        } else {
            self.buffer
                .copy_within(self.cursor..self.end, self.cursor - 1);
            self.cursor -= 1;
            self.end -= 1;
        }

        Ok(())
    }

    pub const fn cursor_left(&mut self) -> Result<()> {
        if self.cursor == 0 {
            return Err(CursorBufferError::CursorAtStart);
        }

        self.cursor -= 1;

        Ok(())
    }

    pub const fn cursor_right(&mut self) -> Result<()> {
        if self.cursor == self.end {
            return Err(CursorBufferError::CursorAtEnd);
        }

        self.cursor += 1;

        Ok(())
    }

    // pub const fn reset(&mut self) {
    //     self.cursor = 0;
    //     self.end = 0;
    // }

    pub const fn is_full(&self) -> bool {
        self.end == SIZE
    }

    pub fn for_each_from_cursor<F>(&self, f: F)
    where
        F: Fn(&T),
    {
        for i in self.cursor..self.end {
            f(&self.buffer[i]);
        }
    }

    pub const fn elements_after_cursor_count(&self) -> usize {
        self.end - self.cursor
    }

    /// Iterates over the buffer, returning a slice for each chunk delimited by the provided closure.
    /// Note: Leading, adjacent and trailing delimiters are consumed.
    pub fn iter_tokens<F>(&self, delimiter: F) -> CursorBufferIterator<SIZE, T, F>
    where
        F: Fn(&T) -> bool,
    {
        // skip leading delimiters
        let mut index = 0;
        while index < self.end && (delimiter)(&self.buffer[index]) {
            index += 1;
        }

        CursorBufferIterator {
            cmd_buf: self,
            index,
            delimiter,
        }
    }
}

pub struct CursorBufferIterator<'a, const SIZE: usize, T, F>
where
    F: Fn(&T) -> bool,
{
    cmd_buf: &'a CursorBuffer<SIZE, T>,
    index: usize,
    delimiter: F,
}

impl<'a, const SIZE: usize, T, F> CursorBufferIterator<'a, SIZE, T, F>
where
    F: Fn(&T) -> bool,
{
    pub fn rest(&self) -> &'a [T] {
        &self.cmd_buf.buffer[self.index..self.cmd_buf.end]
    }
}

impl<'a, const SIZE: usize, T, F> Iterator for CursorBufferIterator<'a, SIZE, T, F>
where
    F: Fn(&T) -> bool,
{
    type Item = &'a [T];

    fn next(&mut self) -> Option<Self::Item> {
        // find the end of the chunk
        let start = self.index;
        while self.index < self.cmd_buf.end && !(self.delimiter)(&self.cmd_buf.buffer[self.index]) {
            self.index += 1;
        }

        // if the chunk is delimiters only, return None
        if start == self.index {
            return None;
        }

        let end = self.index;

        // skip trailing delimiters
        while self.index < self.cmd_buf.end && (self.delimiter)(&self.cmd_buf.buffer[self.index]) {
            self.index += 1;
        }

        Some(&self.cmd_buf.buffer[start..end])
    }
}
