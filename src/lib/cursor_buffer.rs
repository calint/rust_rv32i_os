pub struct CursorBuffer<const SIZE: usize, T> {
    line: [T; SIZE],
    end: usize,
    cursor: usize,
}

impl<const SIZE: usize, T: Default + Copy> CursorBuffer<SIZE, T> {
    pub fn new() -> Self {
        CursorBuffer {
            line: [T::default(); SIZE],
            end: 0,
            cursor: 0,
        }
    }

    pub fn insert(&mut self, ch: T) -> bool {
        if self.end == SIZE {
            return false;
        }

        if self.cursor == self.end {
            self.line[self.cursor] = ch;
            self.cursor += 1;
            self.end += 1;
            return true;
        }

        self.end += 1;
        self.line
            .copy_within(self.cursor..self.end - 1, self.cursor + 1);
        self.line[self.cursor] = ch;
        self.cursor += 1;
        true
    }

    pub fn del(&mut self) {
        if self.cursor == self.end {
            return;
        }

        self.line
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

        self.line
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
            f(self.line[i]);
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
        &self.cmd_buf.line[self.index..self.cmd_buf.end]
    }
}

impl<'a, const SIZE: usize, T, F> Iterator for CursorBufferIterator<'a, SIZE, T, F>
where
    F: Fn(&T) -> bool,
{
    type Item = &'a [T];

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.cmd_buf.end {
            return None;
        }

        // find a delimiter
        let start = self.index;
        self.index += self.cmd_buf.line[self.index..self.cmd_buf.end]
            .iter()
            .position(|c| (self.delimiter)(c))
            .unwrap_or(self.cmd_buf.end - self.index);

        // move forward over the rest of delimiters
        let end = self.index;
        self.index += self.cmd_buf.line[self.index..self.cmd_buf.end]
            .iter()
            .position(|c| !(self.delimiter)(c))
            .unwrap_or(self.cmd_buf.end - self.index);

        Some(&self.cmd_buf.line[start..end])
    }
}
