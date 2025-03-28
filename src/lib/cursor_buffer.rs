pub struct CursorBuffer<const SIZE: usize, T> {
    buffer: [T; SIZE],
    end: usize,
    cursor: usize,
}

impl<const SIZE: usize, T: Default + Copy> CursorBuffer<SIZE, T> {
    pub fn new() -> Self {
        Self {
            buffer: [T::default(); SIZE],
            end: 0,
            cursor: 0,
        }
    }

    pub fn insert(&mut self, ch: T) -> bool {
        if self.end == SIZE {
            return false;
        }

        if self.cursor == self.end {
            self.buffer[self.cursor] = ch;
            self.cursor += 1;
            self.end += 1;
            return true;
        }

        self.end += 1;
        self.buffer
            .copy_within(self.cursor..self.end - 1, self.cursor + 1);
        self.buffer[self.cursor] = ch;
        self.cursor += 1;
        true
    }

    pub fn del(&mut self) {
        if self.cursor == self.end {
            return;
        }

        self.buffer
            .copy_within(self.cursor + 1..self.end, self.cursor);
        self.end -= 1;
    }

    pub fn backspace(&mut self) -> bool {
        if self.cursor == 0 {
            return false;
        }

        if self.cursor == self.end {
            self.end -= 1;
            self.cursor -= 1;
            return true;
        }

        self.buffer
            .copy_within(self.cursor..self.end, self.cursor - 1);
        self.cursor -= 1;
        self.end -= 1;
        true
    }

    pub fn move_cursor_left(&mut self) -> bool {
        if self.cursor == 0 {
            return false;
        }

        self.cursor -= 1;
        true
    }

    pub fn move_cursor_right(&mut self) -> bool {
        if self.cursor == self.end {
            return false;
        }

        self.cursor += 1;
        true
    }

    pub fn reset(&mut self) {
        self.end = 0;
    }

    pub fn is_full(&self) -> bool {
        self.end == SIZE - 1
    }

    pub fn for_each_from_cursor<F>(&self, f: F)
    where
        F: Fn(T),
    {
        for i in self.cursor..self.end {
            f(self.buffer[i]);
        }
    }

    pub fn elements_after_cursor_count(&self) -> usize {
        self.end - self.cursor
    }

    /// Iterate over the buffer returning a slice for each chunk delimited by returning true from delimiter lambda.
    /// Note: Adjacent delimiters are consumed.
    pub fn iter_words<F>(&self, delimiter: F) -> CursorBufferIterator<SIZE, T, F>
    where
        F: Fn(&T) -> bool,
    {
        CursorBufferIterator {
            cmd_buf: self,
            index: 0,
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
        // todo this can be done with only 2 loops by skipping the leading delimiters before creation of iterator
        // skip leading delimiters and find the start of the next chunk
        while self.index < self.cmd_buf.end && (self.delimiter)(&self.cmd_buf.buffer[self.index]) {
            self.index += 1;
        }

        if self.index >= self.cmd_buf.end {
            return None;
        }

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
