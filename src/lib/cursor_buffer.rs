//
// reviewed: 2025-04-21
//
pub struct CursorBuffer<const SIZE: usize, T> {
    buffer: [T; SIZE],
    end: usize,
    cursor: usize,
}

pub type Result<T> = core::result::Result<T, Error>;

pub enum Error {
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
            return Err(Error::BufferFull);
        }

        if self.cursor == self.end {
            self.buffer[self.cursor] = ch;
            self.cursor += 1;
            self.end += 1;

            return Ok(());
        }

        self.end += 1;
        self.buffer
            .copy_within(self.cursor..self.end - 1, self.cursor + 1);
        self.buffer[self.cursor] = ch;
        self.cursor += 1;

        Ok(())
    }

    pub fn delete(&mut self) -> Result<()> {
        if self.cursor == self.end {
            return Err(Error::CursorAtEnd);
        }

        self.buffer
            .copy_within(self.cursor + 1..self.end, self.cursor);
        self.end -= 1;

        Ok(())
    }

    pub fn backspace(&mut self) -> Result<()> {
        if self.cursor == 0 {
            return Err(Error::CursorAtStart);
        }

        if self.cursor == self.end {
            self.end -= 1;
            self.cursor -= 1;

            return Ok(());
        }

        self.buffer
            .copy_within(self.cursor..self.end, self.cursor - 1);
        self.cursor -= 1;
        self.end -= 1;

        Ok(())
    }

    /// Returns the number of character positions the cursor moved.
    pub const fn move_cursor_left(&mut self) -> usize {
        if self.cursor == 0 {
            return 0;
        }

        self.cursor -= 1;

        1
    }

    /// Returns the number of character positions the cursor moved.
    pub const fn move_cursor_right(&mut self) -> usize {
        if self.cursor == self.end {
            return 0;
        }

        self.cursor += 1;

        1
    }

    /// Returns the number of character positions the cursor moved to reach the start of the line.
    pub const fn move_cursor_to_start_of_line(&mut self) -> usize {
        let moves = self.cursor;
        self.cursor = 0;

        moves
    }

    /// Returns the number of character positions the cursor moved to reach the end of the line.
    pub const fn move_cursor_to_end_of_line(&mut self) -> usize {
        let moves = self.end - self.cursor;
        self.cursor = self.end;

        moves
    }

    pub const fn is_full(&self) -> bool {
        self.end == SIZE
    }

    // Applies `f` on each element from cursor to end.
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
    // Returns the rest of elements as a slice.
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
