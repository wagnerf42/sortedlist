//! Implement python SortedList from sortedcontainers.

/// Python's SortedList structure.
/// A kind of flat BTree.
/// If you choose a block size of sqrt(n) you get all operations
/// in amortized O(n**(1/3)).
pub struct SortedList<T> {
    indexes: Vec<T>,
    data: Vec<Vec<T>>,
    block_size: usize,
}

impl<T: Copy + Ord> SortedList<T> {
    /// Create a new `SortedList` with given block size.
    pub fn new(block_size: usize) -> Self {
        SortedList {
            indexes: Vec::new(),
            data: Vec::new(),
            block_size,
        }
    }

    /// Iterate in order on all elements contained.
    pub fn iter<'a>(&'a self) -> impl Iterator<Item = &'a T> + 'a {
        self.data.iter().flatten()
    }

    /// Return if we contain given value.
    /// This runs in O(log(n)) whatever the block size.
    pub fn contains(&self, value: &T) -> bool {
        let block_index = match self.indexes.binary_search(value) {
            Ok(_) => return true,
            Err(i) => i,
        };
        self.data
            .get(block_index)
            .and_then(|b| b.binary_search(value).ok())
            .is_some()
    }

    /// Insert element at given position.
    pub fn insert(&mut self, element: T) {
        let mut target_block = match self.indexes.binary_search(&element) {
            Ok(i) => i,
            Err(i) => i,
        };
        if target_block == self.data.len() {
            if target_block == 0 {
                // first insert is a special case
                self.indexes.push(element);
                let mut new_vec = Vec::with_capacity(self.block_size);
                new_vec.push(element);
                self.data.push(new_vec);
                return;
            }
            target_block -= 1;
        }

        if self.data[target_block].len() == self.block_size {
            self.rebalance(target_block);
            if self.indexes[target_block] <= element {
                target_block += 1;
            }
        }

        let block = &mut self.data[target_block];
        let target_position = match block.binary_search(&element) {
            Ok(i) => i,
            Err(i) => i,
        };
        if target_position == block.len() {
            self.indexes[target_block] = element;
        }
        block.insert(target_position, element);
    }

    fn rebalance(&mut self, block_index: usize) {
        let mid = self.block_size / 2;
        //        let mut new_vec = Vec::with_capacity(self.block_size);
        //        new_vec.extend(self.data[block_index].drain(mid..));
        let new_vec = self.data[block_index].drain(mid..).collect::<Vec<_>>();
        self.data.insert(block_index + 1, new_vec);
        self.indexes
            .insert(block_index, self.data[block_index].last().cloned().unwrap());
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
}
