use std::fmt::{Debug, Display};

struct SimpleRingbuf<V> {
    data: Vec<V>,
    start: usize,
    end: usize,
    size: usize,
    full: bool,
}

impl<T> SimpleRingbuf<T>
where
    T: Default + Display + Debug,
{
    fn new(size: usize) -> SimpleRingbuf<T> {
        let mut data = Vec::with_capacity(size);
        for _i in 0..size {
            data.push(Default::default());
        }
        println!("data: {:?}", data);
        SimpleRingbuf {
            data,
            start: 0,
            end: 0,
            size,
            full: false,
        }
    }

    fn push(&mut self, value: T) {
        self.data[self.end] = value;
        self.end = (self.end + 1) % self.size;
        if self.end == self.start {
            self.start = (self.start + 1) % self.size;
            self.full = true;
        }
    }
    fn iter(&self) -> SimpleRingbufIter<T> {
        let index = match self.full {
            // In the case where the buffer is full (i.e. we have valid data everywhere and are
            // wrapping around), we need to start iterating at start-1, because start will always
            // lead end by one, this means start-1 = end, so we can use that.
            // It would not be valid to use start-1 before we've filled the first time as that
            // would wrap back to unwritten data.
            false => self.start,
            true => self.end,
        };
        SimpleRingbufIter {
            ringbuf: self,
            index,
            empty: self.start == self.end,
            count: 0,
        }
    }
}

impl<T> SimpleRingbuf<T>
where
    T: Clone + Default + Display + Debug,
{
    /// Same as [push] but emits the replaced T if it was so
    fn push_emit(&mut self, value: T) -> Option<T> {
        let to_return = if self.full {
            // when full, self.end == self.start - 1
            Some(self.data[self.end].clone())
        } else {
            None
        };
        self.push(value);
        to_return
    }

    fn flatten_copy(&self) -> Vec<T> {
        self.iter().map(|v| v.clone()).collect()
    }
}

struct SimpleRingbufIter<'a, T> {
    ringbuf: &'a SimpleRingbuf<T>,
    index: usize,
    empty: bool,
    count: usize,
}

impl<'a, T> Iterator for SimpleRingbufIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.empty {
            return None;
        }
        if self.index == self.ringbuf.end && self.count > 0 {
            return None;
        }
        let value = &self.ringbuf.data[self.index];
        self.index = (self.index + 1) % self.ringbuf.size;
        self.count += 1;
        Some(value)
    }
}

struct HistoryStore<T> {
    primary_ringbuf: SimpleRingbuf<T>,
    storage_ringbuf: SimpleRingbuf<T>,
    persistence_method: PersistenceMethod,
}

impl<T: Default + Display + Debug + Clone> HistoryStore<T> {
    fn new(primary_size: usize, storage_size: usize, store_every_nth: usize) -> HistoryStore<T> {
        Self {
            primary_ringbuf: SimpleRingbuf::new(primary_size),
            storage_ringbuf: SimpleRingbuf::new(storage_size),
            persistence_method: PersistenceMethod::KeepEvery(KeepEveryMethod::new(store_every_nth)),
        }
    }

    fn push(&mut self, value: T) {
        match self.primary_ringbuf.push_emit(value) {
            Some(replaced) => {
                if self.persistence_method.persist_next() {
                    self.storage_ringbuf.push(replaced);
                }
            }
            None => {}
        }
    }
}

enum PersistenceMethod {
    KeepEvery(KeepEveryMethod),
}

impl PersistenceMethod {
    fn persist_next(&mut self) -> bool {
        match self {
            PersistenceMethod::KeepEvery(method) => method.persist_next(),
        }
    }
}

struct KeepEveryMethod {
    nth: usize,
    count: usize,
}

impl KeepEveryMethod {
    fn new(nth: usize) -> KeepEveryMethod {
        KeepEveryMethod { nth, count: 0 }
    }

    fn persist_next(&mut self) -> bool {
        self.count = (self.count + 1) % self.nth;
        self.count == 0
    }
}

mod test {
    use super::*;

    #[test]
    fn test_ringbuf() {
        let mut ringbuf: SimpleRingbuf<u8> = SimpleRingbuf::new(5);
        ringbuf.push(1);
        ringbuf.push(2);
        ringbuf.push(3);
        assert_eq!(ringbuf.data, vec![1, 2, 3, 0, 0]);
        let ordered_data = ringbuf.flatten_copy();
        assert_eq!(ordered_data, vec![1, 2, 3]);
        ringbuf.push(4);
        ringbuf.push(5);
        ringbuf.push(6);
        assert_eq!(ringbuf.data, vec![6, 2, 3, 4, 5]);
        assert_eq!(ringbuf.flatten_copy(), vec![2, 3, 4, 5, 6]);
        ringbuf.push(7);
        ringbuf.push(8);
        ringbuf.push(9);
        ringbuf.push(10);
        assert_eq!(ringbuf.data, vec![6, 7, 8, 9, 10]);
        assert_eq!(ringbuf.flatten_copy(), vec![6, 7, 8, 9, 10]);
    }

    #[test]
    fn test_storage() {
        let mut history_store: HistoryStore<u8> = HistoryStore::new(3, 10, 3);
        for i in 1..=10 {
            history_store.push(i);
        }
        assert_eq!(history_store.primary_ringbuf.flatten_copy(), vec![8, 9, 10]);
        assert_eq!(history_store.storage_ringbuf.flatten_copy(), vec![3, 6]);
    }
}
