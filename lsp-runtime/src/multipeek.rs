use std::collections::VecDeque;

pub struct MultiPeek<I: Iterator> {
    inner: I,
    peek_buffer: VecDeque<I::Item>,
}

impl<I: Iterator> Iterator for MultiPeek<I> {
    type Item = I::Item;
    fn next(&mut self) -> Option<Self::Item> {
        if self.peek_buffer.is_empty() {
            self.inner.next()
        } else {
            self.peek_buffer.pop_front()
        }
    }
}

impl<I: Iterator> MultiPeek<I> {
    pub fn from_iter(iter: I) -> Self {
        Self {
            inner: iter,
            peek_buffer: VecDeque::new(),
        }
    }
    #[inline(always)]
    pub fn peek_n(&mut self, n: usize) -> Option<&I::Item> {
        while self.peek_buffer.len() < n {
            if let Some(item) = self.inner.next() {
                self.peek_buffer.push_back(item);
            } else {
                return None;
            }
        }
        if n > 0 {
            self.peek_buffer.get(n - 1)
        } else {
            None
        }
    }
    #[inline(always)]
    pub fn peek(&mut self) -> Option<&I::Item> {
        self.peek_n(1)
    }

    pub fn peek_fold<U, F>(&mut self, init: U, mut func: F) -> U
    where
        F: FnMut(&U, &I::Item) -> Option<U>,
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

#[cfg(test)]
mod test {
    use crate::MultiPeek;
    #[test]
    fn test_iter_api() {
        let inner: Vec<_> = (0..1000).collect();
        let mut mp_iter = MultiPeek::from_iter(inner.clone().into_iter());
        assert_eq!(mp_iter.peek(), Some(&0));
        assert_eq!(inner.into_iter().sum::<i32>(), mp_iter.sum());
    }
    #[test]
    fn test_peek_out_of_bound() {
        let inner: Vec<_> = (0..1000).collect();
        let mut mp_iter = MultiPeek::from_iter(inner.clone().into_iter());
        assert_eq!(mp_iter.peek_n(1001), None);
        assert_eq!(mp_iter.peek_n(1000), Some(&999));
        assert_eq!(mp_iter.peek_n(1002), None);
        assert_eq!(mp_iter.peek_n(5002), None);
    }
    #[test]
    fn test_peek_empty_inner_iter() {
        let inner: Vec<_> = (0..0).collect();
        let mut mp_iter = MultiPeek::from_iter(inner.clone().into_iter());
        assert_eq!(mp_iter.peek_n(0), None);
        assert_eq!(mp_iter.peek_n(1), None);
        assert_eq!(mp_iter.peek_n(1000), None);
    }
    #[test]
    fn test_peek_fold_full() {
        let inner: Vec<_> = (0..1000).collect();
        let mut mp_iter = MultiPeek::from_iter(inner.clone().into_iter());
        assert_eq!(
            mp_iter.peek_fold(0, |a, b| Some(a + b)),
            inner.into_iter().sum::<i32>()
        );
        assert_eq!(mp_iter.next(), Some(0));
    }
    #[test]
    fn test_peek_fold_early_terminate() {
        let inner: Vec<_> = (0..1000).collect();
        let mut mp_iter = MultiPeek::from_iter(inner.clone().into_iter());
        assert_eq!(
            mp_iter.peek_fold(0, |a, b| if *b < 500 { Some(a + b) } else { None }),
            inner.into_iter().filter(|&x| x < 500).sum::<i32>()
        );
        assert_eq!(mp_iter.next(), Some(0));
    }
}
