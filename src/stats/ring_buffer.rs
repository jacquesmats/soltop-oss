/// A fixed-size circular buffer that overwrites old data when full.
///
/// Generic over type T - can store any data (numbers, structs, etc.)
pub struct RingBuffer<T> {
    data: Vec<T>,
    capacity: usize,
    head: usize,
    count: usize,
}

impl<T> RingBuffer<T> {
    /// Creates a new ring buffer with the given capacity.
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "RingBuffer capacity must be greater than 0");
        Self {
            data: Vec::with_capacity(capacity),
            capacity,
            head: 0,
            count: 0,
        }
    }

    /// If full, overwrites the oldest item.
    pub fn push(&mut self, item: T) {
        if self.count < self.capacity {
            self.data.push(item);
            self.count += 1;
        } else if self.head < self.capacity {
            self.data[self.head] = item;
        }
        // Move head forward, wrap around
        self.head = (self.head + 1) % self.capacity;
    }

    /// Returns an iterator over the buffer's items in insertion order.
    /// Oldest items first, newest items last.
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        if self.is_full() {
            self.data[self.head..]
                .iter()
                .chain(self.data[..self.head].iter())
        } else {
            self.data[0..self.count].iter().chain([].iter())
        }
    }

    /// Returns the current number of items in the buffer.
    pub fn len(&self) -> usize {
        self.count
    }

    /// Returns true if the buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Returns the maximum capacity.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Returns true if the buffer is at capacity.
    pub fn is_full(&self) -> bool {
        self.count == self.capacity
    }

    /// Removes all items from the buffer.
    pub fn clear(&mut self) {
        // TODO: Clear Vec and reset head/count
        self.data.clear();
        self.count = 0;
        self.head = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_buffer() {
        let buffer: RingBuffer<i32> = RingBuffer::new(5);

        assert_eq!(buffer.len(), 0);
        assert_eq!(buffer.capacity(), 5);
        assert!(buffer.is_empty());
        assert!(!buffer.is_full());
    }

    #[test]
    fn test_push_to_capacity() {
        // TODO: Test pushing items until full : RingBuffer
        let mut buffer: RingBuffer<i32> = RingBuffer::new(3);

        buffer.push(1);
        buffer.push(2);
        buffer.push(3);

        assert!(!buffer.is_empty());
        assert!(buffer.is_full());
        assert_eq!(buffer.len(), 3);
    }

    #[test]
    fn test_push_overwrites() {
        // TODO: Test that pushing beyond capacity overwrites
        let mut buffer: RingBuffer<i32> = RingBuffer::new(3);
        buffer.push(1);
        buffer.push(2);
        buffer.push(3);
        buffer.push(4);
        buffer.push(5);
        // Push 1, 2, 3, 4, 5
        // Assert buffer contains [4, 5, 3] or similar
        assert_eq!(buffer.len(), 3);

        let items: Vec<_> = buffer.iter().collect();
        assert_eq!(items, vec![&3, &4, &5]);
    }

    #[test]
    fn test_iter_not_full() {
        // TODO: Test iteration when buffer not full
        let mut buffer: RingBuffer<i32> = RingBuffer::new(5);
        buffer.push(1);
        buffer.push(2);
        buffer.push(3);

        let iter: Vec<_> = buffer.iter().collect();
        assert_eq!(iter, vec![&1, &2, &3]);
        assert!(!iter.is_empty());
    }

    #[test]
    fn test_iter_wrapped() {
        // TODO: Test iteration when buffer has wrapped
        let mut buffer: RingBuffer<i32> = RingBuffer::new(3);
        buffer.push(1);
        buffer.push(2);
        buffer.push(3);
        buffer.push(4);
        buffer.push(5);

        let iter: Vec<&i32> = buffer.iter().collect();
        assert_eq!(iter, vec![&3, &4, &5]);
    }

    #[test]
    fn test_clear() {
        let mut buffer = RingBuffer::new(3);
        buffer.push(1);
        buffer.push(2);

        buffer.clear();

        assert_eq!(buffer.len(), 0);
        assert!(buffer.is_empty());
        // TODO: Test clear() resets buffer
    }

    #[test]
    #[should_panic(expected = "RingBuffer capacity must be greater than 0")]
    fn test_zero_capacity_panics() {
        // TODO: Test that new(0) panics
        let _buffer: RingBuffer<i32> = RingBuffer::new(0);
    }
}
