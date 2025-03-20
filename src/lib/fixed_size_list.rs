// Define the FixedSizeList struct
#[derive(Copy, Clone, PartialEq)]
pub struct FixedSizeList<T, const N: usize> {
    data: [Option<T>; N],
    count: usize,
}

impl<T: Copy + PartialEq, const N: usize> FixedSizeList<T, N> {
    pub fn new() -> Self {
        FixedSizeList {
            data: [None; N],
            count: 0,
        }
    }

    pub fn add(&mut self, item: T) -> bool {
        if self.count < N {
            self.data[self.count] = Some(item);
            self.count += 1;
            true
        } else {
            false
        }
    }

    pub fn remove(&mut self, item: T) -> bool {
        for i in 0..self.count {
            if self.data[i] == Some(item) {
                return self.remove_at(i);
            }
        }
        false
    }

    pub fn remove_at(&mut self, index: usize) -> bool {
        if index < self.count {
            self.data[index] = None;
            for i in index..self.count - 1 {
                self.data[i] = self.data[i + 1];
            }
            self.count -= 1;
            self.data[self.count] = None;
            true
        } else {
            false
        }
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        if index < self.count {
            self.data[index].as_ref()
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index < self.count {
            self.data[index].as_mut()
        } else {
            None
        }
    }

    pub fn iter(&self) -> FixedSizeListIterator<T, N> {
        FixedSizeListIterator {
            list: self,
            index: 0,
        }
    }
}

pub struct FixedSizeListIterator<'a, T, const N: usize> {
    list: &'a FixedSizeList<T, N>,
    index: usize,
}

impl<'a, T: Copy, const N: usize> Iterator for FixedSizeListIterator<'a, T, N> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.list.count {
            let item = self.list.data[self.index];
            self.index += 1;
            item
        } else {
            None
        }
    }
}
