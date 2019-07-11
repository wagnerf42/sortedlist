//! Implement python SortedList from sortedcontainers.
use std::borrow::Borrow;

/// Python's SortedList structure.
/// A kind of flat BTree.
/// If you choose a block size of sqrt(n) you get all operations
/// in amortized O(n**(1/3)).
pub struct SortedList<T> {
    data: Vec<Vec<T>>,
    block_size: usize,
}

impl<T: Ord> SortedList<T> {
    /// Create a new `SortedList` with given block size.
    pub fn new(block_size: usize) -> Self {
        SortedList {
            data: Vec::new(),
            block_size,
        }
    }

    /// Iterate in order on all elements contained.
    pub fn iter<'a>(&'a self) -> impl Iterator<Item = &'a T> + 'a {
        self.data.iter().flatten()
    }

    /// Remove given element (any). Return true if it was here.
    pub fn remove<Q>(&mut self, value: &Q) -> bool
    where
        Q: Ord + ?Sized,
        T: Borrow<Q>,
    {
        if let Some((block_index, element_index)) = self.indexes_for(value) {
            self.data[block_index].remove(element_index);
            let block_len = self.data[block_index].len();
            if block_index > 0 && block_len < self.block_size / 2 {
                // we are not big enough, we should fuse with previous block
                // two cases: whether we end with one or two buffers.
                let cumulated_size = self.data[block_index - 1].len() + block_len;
                if cumulated_size <= self.block_size {
                    // easy case, just append current block at end of previous one
                    let to_redispatch = self.data.remove(block_index);
                    self.data[block_index - 1].extend(to_redispatch);
                } else {
                    // hard case, we need to redispatch some of previous buffer's in us.
                    let target_size = cumulated_size / 2;
                    let moved_size = self.data[block_index - 1].len() - target_size;
                    unsafe {
                        // move data back at end of vector
                        let buffer = &mut self.data[block_index][0] as *mut T;
                        let end = buffer.offset(moved_size as isize);
                        buffer.copy_to(end, block_len);
                        self.data[block_index].set_len(block_len + moved_size);
                        // move data from end of previous vector here
                        let previous_data = &self.data[block_index - 1][target_size] as *const T;
                        previous_data.copy_to_nonoverlapping(buffer, moved_size);
                        self.data[block_index - 1].set_len(target_size);
                    }
                }
            }
            true
        } else {
            false
        }
    }

    fn block_index<Q>(&self, value: &Q) -> usize
    where
        Q: Ord + ?Sized,
        T: Borrow<Q>,
    {
        // note : this code is copy pasted from the slice's binary search in standard library.
        let mut size = self.data.len();
        if size == 0 {
            return 0;
        }
        let mut base = 0usize;
        while size > 1 {
            let half = size / 2;
            let mid = base + half;
            // mid is always in [0, size), that means mid is >= 0 and < size.
            // mid >= 0: by definition
            // mid < size: mid = size / 2 + size / 4 + size / 8 ...
            let cmp = value
                .cmp(unsafe { self.data[mid].get_unchecked(self.data[mid].len() - 1) }.borrow());
            base = if cmp == std::cmp::Ordering::Greater {
                mid
            } else {
                base
            };
            size -= half;
        }
        // base is always in [0, size) because base <= mid.
        let cmp =
            value.cmp(unsafe { self.data[base].get_unchecked(self.data[base].len() - 1) }.borrow());
        if cmp == std::cmp::Ordering::Equal {
            base
        } else {
            base + (cmp == std::cmp::Ordering::Greater) as usize
        }
    }

    /// Return block index and index in block for given value.
    fn indexes_for<Q>(&self, value: &Q) -> Option<(usize, usize)>
    where
        Q: Ord + ?Sized,
        T: Borrow<Q>,
    {
        let block_index = self.block_index(value);
        self.data
            .get(block_index)
            .and_then(|b| b.binary_search_by_key(&value, |t| t.borrow()).ok())
            .map(|i| (block_index, i))
    }

    /// Return if we contain given value.
    /// This runs in O(log(n)) whatever the block size.
    pub fn contains<Q>(&self, value: &Q) -> bool
    where
        Q: Ord + ?Sized,
        T: Borrow<Q>,
    {
        let block_index = self.block_index(value);
        self.data
            .get(block_index)
            .and_then(|b| b.binary_search_by_key(&value, |t| t.borrow()).ok())
            .is_some()
    }

    /// Insert element at given position.
    pub fn insert(&mut self, element: T) {
        let mut target_block = self.block_index(&element);
        if target_block == self.data.len() {
            if target_block == 0 {
                // first insert is a special case
                let mut new_vec = Vec::with_capacity(self.block_size);
                new_vec.push(element);
                self.data.push(new_vec);
                return;
            }
            target_block -= 1;
        }

        if self.data[target_block].len() == self.block_size {
            self.rebalance(target_block);
            if *self.data[target_block].last().unwrap() <= element {
                target_block += 1;
            }
        }

        let block = &mut self.data[target_block];
        let target_position = match block.binary_search(&element) {
            Ok(i) => i,
            Err(i) => i,
        };
        block.insert(target_position, element);
    }

    fn rebalance(&mut self, block_index: usize) {
        let mid = self.data[block_index].len() / 2;
        let mut new_vec = Vec::with_capacity(self.block_size);
        new_vec.extend(self.data[block_index].drain(mid..));
        self.data.insert(block_index + 1, new_vec);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn insert_range() {
        let mut l = SortedList::new(1_000);
        for x in 0..1_000_000 {
            l.insert(x);
        }
        assert!(l.iter().cloned().eq(0..1_000_000));
    }
    #[test]
    fn insert_reversed_range() {
        let mut l = SortedList::new(1_000);
        for x in (0..1_000_000).rev() {
            l.insert(x);
        }
        assert!(l.iter().cloned().eq(0..1_000_000));
    }
    #[test]
    fn contains() {
        let mut l = SortedList::new(1_000);
        for x in (0..1_000_000).rev() {
            l.insert(x);
        }
        assert!(l.contains(&500_000));
        assert!(!l.contains(&1_000_000));
    }
    #[test]
    fn remove() {
        let mut l = SortedList::new(1_000);
        for x in (0..1_000_000).rev() {
            l.insert(x);
        }
        for x in (0..1_000_000).filter(|&x| x % 7 == 0) {
            assert!(l.remove(&x));
        }
        assert!(l.iter().cloned().eq((0..1_000_000).filter(|&x| x % 7 != 0)));
    }
}
