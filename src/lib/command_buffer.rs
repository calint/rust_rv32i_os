pub struct CommandBuffer<const SIZE: usize, T> {
    line: [T; SIZE],
    end: usize,
    cursor: usize,
}

impl<const SIZE: usize, T: Default + Copy> CommandBuffer<SIZE, T> {
    pub fn new() -> Self {
        CommandBuffer {
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
        for i in (self.cursor + 1..=self.end).rev() {
            self.line[i] = self.line[i - 1];
        }

        self.line[self.cursor] = ch;
        self.cursor += 1;
        true
    }

    pub fn del(&mut self) {
        if self.cursor == self.end {
            return;
        }

        for i in self.cursor + 1..self.end {
            self.line[i - 1] = self.line[i];
        }

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

        for i in self.cursor - 1..self.end - 1 {
            self.line[i] = self.line[i + 1];
        }

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

    pub fn apply_on_elements_from_cursor_to_end<F>(&self, mut f: F)
    where
        F: FnMut(T),
    {
        for i in self.cursor..self.end {
            f(self.line[i]);
        }
    }

    pub fn elements_after_cursor_count(&self) -> usize {
        self.end - self.cursor
    }

    // iterate over the buffer returning a slice for each word
    pub fn iter_words(&self) -> CommandBufferIterator<SIZE, T> {
        CommandBufferIterator {
            cmd_buf: self,
            index: 0,
        }
    }
}

// iterator over the command buffer returning a slice for each word
pub struct CommandBufferIterator<'a, const SIZE: usize, T> {
    cmd_buf: &'a CommandBuffer<SIZE, T>,
    index: usize,
}

impl<'a, const SIZE: usize, T> CommandBufferIterator<'a, SIZE, T> {
    pub fn rest(&self) -> &'a [T] {
        &self.cmd_buf.line[self.index..self.cmd_buf.end]
    }
}

impl<'a, const SIZE: usize> Iterator for CommandBufferIterator<'a, SIZE, u8> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.cmd_buf.end {
            let start = self.index;
            while self.index < self.cmd_buf.end
                && !self.cmd_buf.line[self.index].is_ascii_whitespace()
            {
                self.index += 1;
            }
            let end = self.index;
            while self.index < self.cmd_buf.end
                && self.cmd_buf.line[self.index].is_ascii_whitespace()
            {
                self.index += 1;
            }
            Some(&self.cmd_buf.line[start..end])
        } else {
            None
        }
    }
}
