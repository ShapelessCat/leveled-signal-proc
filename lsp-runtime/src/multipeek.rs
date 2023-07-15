use std::collections::VecDeque;


pub struct MultiPeek<I:Iterator> {
    inner: I,
    peek_buffer: VecDeque<I::Item>,
}

impl <I:Iterator> Iterator for MultiPeek<I> {
    type Item = I::Item;
    fn next(&mut self) -> Option<Self::Item> {
        if self.peek_buffer.is_empty() {
            self.inner.next()
        } else {
            self.peek_buffer.pop_front()
        }
    }
}

impl <I:Iterator> MultiPeek<I> {
    pub fn from_iter(iter: I) -> Self {
        Self { inner: iter, peek_buffer: VecDeque::new() }
    }
    pub fn peek_n(&mut self, n: usize) -> Option<&I::Item> {
        while self.peek_buffer.len() < n {
            if let Some(item) = self.inner.next() {
                self.peek_buffer.push_back(item);
            } else {
                return None;
            }
        }
        self.peek_buffer.get(n - 1)
    }

    pub fn peek(&mut self) -> Option<&I::Item> {
        self.peek_n(1)
    }

    pub fn peek_fold<U, F>(&mut self, init: U, mut func: F) -> U
    where
        F: FnMut(&U, &I::Item) -> Option<U>
    {
        let mut ret = init;
        for i in 1.. {
            if let Some(item) = self.peek_n(i) {
                if let Some(new_value) = func(&ret, item) {
                    ret = new_value;
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        ret
    }
}