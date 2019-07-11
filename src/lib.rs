//! Implement python SortedList from sortedcontainers.
use std::borrow::Borrow;

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

    /// Remove given element (any). Return true if it was here.
    pub fn remove<Q>(&mut self, value: &Q) -> bool
    where
        Q: Ord + ?Sized,
        T: Borrow<Q>,
    {
        if let Some((block_index, element_index)) = self.indexes_for(value) {
            self.data[block_index].remove(element_index);
            if element_index == self.data[block_index].len() + 1 {
                // if we were the max of the block we need to update indices
                if element_index > 0 {
                    self.indexes[block_index] = self.data[block_index].last().cloned().unwrap();
                }
            }
            if block_index > 0 && self.data[block_index].len() < self.block_size / 2 {
                // we are not big enough, we should fuse with previous block
                let mut to_redispatch = self.data.remove(block_index);
                self.indexes.remove(block_index);
                self.data[block_index - 1].extend(to_redispatch.drain(..));
                self.indexes[block_index - 1] = self.data[block_index - 1].last().cloned().unwrap();
                if self.data[block_index - 1].len() > self.block_size {
                    // TODO: we could do better and avoid removing and repushing stuff
                    self.rebalance(block_index - 1);
                }
            }
            true
        } else {
            false
        }
    }

    /// Return block index and index in block for given value.
    fn indexes_for<Q>(&self, value: &Q) -> Option<(usize, usize)>
    where
        Q: Ord + ?Sized,
        T: Borrow<Q>,
    {
        let block_index = match self.indexes.binary_search_by_key(&value, |t| t.borrow()) {
            Ok(i) => return Some((i, self.data[i].len() - 1)),
            Err(i) => i,
        };
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
        let block_index = match self.indexes.binary_search_by_key(&value, |t| t.borrow()) {
            Ok(_) => return true,
            Err(i) => i,
        };
        self.data
            .get(block_index)
            .and_then(|b| b.binary_search_by_key(&value, |t| t.borrow()).ok())
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
        let mid = self.data[block_index].len() / 2;
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
    #[test]
    fn remove() {
        let mut l = SortedList::new(1_000);
        for x in (0..1_000_000).rev() {
            l.insert(x);
        }
        for x in (0..1_000_000).filter(|&x| x % 2 == 0) {
            assert!(l.remove(&x));
        }
        assert!(l.iter().cloned().eq((0..1_000_000).filter(|&x| x % 2 == 1)));
    }
}
