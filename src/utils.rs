use std::collections::VecDeque;

pub struct FixedVecDeque<T> {
    deque: VecDeque<T>,
    max_len: usize,
}

impl<T> FixedVecDeque<T> {
    pub fn new(max_len: usize) -> Self {
        Self {
            deque: VecDeque::new(),
            max_len,
        }
    }

    pub fn push(&mut self, value: T) {
        if self.deque.len() == self.max_len {
            self.deque.pop_front();
        }
        self.deque.push_back(value);
    }

    pub fn as_deque(&self) -> &VecDeque<T> {
        &self.deque
    }

    pub fn len(&self) -> usize {
        self.deque.len()
    }
}
