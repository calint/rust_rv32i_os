use core::usize;

type Gen = usize;

pub struct Key {
    index: usize,
    generation: Gen,
}

struct Entry<T> {
    generation: Gen,
    item: T,
}

pub struct GenList<T, const N: usize> {
    items: [Option<Entry<T>>; N],
    items_end: usize,
    free_slots: [usize; N],
    free_slots_end: usize,
    generation: Gen,
}

impl<T, const N: usize> GenList<T, N> {
    pub fn new() -> Self {
        GenList {
            items: [const { None }; N],
            items_end: 0,
            free_slots: [0; N],
            free_slots_end: 0,
            generation: 0,
        }
    }

    pub fn insert(&mut self, item: T) -> Option<Key> {
        let ix = if self.free_slots_end == 0 {
            if self.items_end == self.items.len() {
                return None;
            }
            let ix = self.items_end;
            self.items_end += 1;
            ix
        } else {
            self.free_slots_end -= 1;
            let ix = self.free_slots[self.free_slots_end];
            ix
        };
        self.generation += 1;
        self.items[ix] = Some(Entry {
            generation: self.generation,
            item: item,
        });
        Some(Key {
            index: ix,
            generation: self.generation,
        })
    }

    pub fn get(&self, key: &Key) -> Option<&T> {
        match &self.items[key.index] {
            Some(entry) => {
                if entry.generation != key.generation {
                    None
                } else {
                    Some(&entry.item)
                }
            }
            None => None,
        }
    }

    pub fn get_mut(&mut self, key: &Key) -> Option<&mut T> {
        match &mut self.items[key.index] {
            Some(entry) => {
                if entry.generation != key.generation {
                    None
                } else {
                    Some(&mut entry.item)
                }
            }
            None => None,
        }
    }

    pub fn remove(&mut self, key: &Key) -> bool {
        match &self.items[key.index] {
            Some(entry) => {
                if entry.generation != key.generation {
                    false
                } else {
                    self.items[key.index] = None;
                    self.free_slots[self.free_slots_end] = key.index;
                    self.free_slots_end += 1;
                    true
                }
            }
            None => false,
        }
    }
}
