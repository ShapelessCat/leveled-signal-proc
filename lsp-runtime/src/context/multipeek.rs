use std::collections::VecDeque;

use serde::{Deserialize, Serialize};

use crate::signal_api::Patchable;

fn serialize_vecdeque_len<T, S>(vec: &VecDeque<T>, s: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    s.serialize_u64(vec.len() as u64)
}

#[derive(Serialize)]
#[serde(bound = "")]
pub struct MultiPeek<I: Iterator> {
    #[serde(skip_serializing)]
    inner: I,
    offset: usize,
    #[serde(rename = "peek_buffer_size", serialize_with = "serialize_vecdeque_len")]
    peek_buffer: VecDeque<I::Item>,
}

impl<I: Iterator> Iterator for MultiPeek<I> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if self.peek_buffer.is_empty() {
            // TODO: how to make sure the following two consecutive statements execute atomically?
            self.offset += 1;
            self.inner.next()
        } else {
            self.peek_buffer.pop_front()
        }
    }
}

impl<I: Iterator> From<I> for MultiPeek<I> {
    fn from(inner: I) -> Self {
        Self {
            inner,
            offset: 0usize,
            peek_buffer: VecDeque::new(),
        }
    }
}

#[derive(Default, Deserialize)]
pub struct MultiPeekState {
    offset: usize,
    peek_buffer_size: usize,
}

impl<I: Iterator> Patchable for MultiPeek<I> {
    type State = MultiPeekState;

    // TODO: Make this more robust, and handle the issue: `next()` result is `None`.
    //       This shouldn't happen when the iterator is built from the original data source.
    fn patch_from(&mut self, state: Self::State) {
        self.offset = state.offset;
        let n4drop = state.offset - state.peek_buffer_size;
        (0..n4drop).for_each(|_| {
            self.inner.next();
        });
        (0..state.peek_buffer_size)
            .for_each(|_| self.peek_buffer.push_back(self.inner.next().unwrap()));
    }
}

impl<I: Iterator> MultiPeek<I> {
    #[inline(always)]
    pub fn offset(&self) -> usize {
        self.offset
    }

    #[inline(always)]
    pub fn peek_n(&mut self, n: usize) -> Option<&I::Item> {
        while self.peek_buffer.len() < n {
            if let Some(item) = self.inner.next() {
                // TODO: how to make sure the following two consecutive statements execute atomically?
                self.offset += 1;
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
    use super::MultiPeek;

    #[test]
    fn test_iter_api() {
        let inner: Vec<_> = (0..1000).collect();
        let mut mp_iter = MultiPeek::from(inner.clone().into_iter());
        assert_eq!(mp_iter.peek(), Some(&0));
        assert_eq!(inner.into_iter().sum::<i32>(), mp_iter.sum::<i32>());
    }
    #[test]
    fn test_peek_out_of_bound() {
        let inner: Vec<_> = (0..1000).collect();
        let mut mp_iter = MultiPeek::from(inner.clone().into_iter());
        assert_eq!(mp_iter.peek_n(1001), None);
        assert_eq!(mp_iter.peek_n(1000), Some(&999));
        assert_eq!(mp_iter.peek_n(1002), None);
        assert_eq!(mp_iter.peek_n(5002), None);
    }
    #[test]
    fn test_peek_empty_inner_iter() {
        let inner: Vec<_> = (0..0).collect();
        let mut mp_iter = MultiPeek::from(inner.clone().into_iter());
        assert_eq!(mp_iter.peek_n(0), None);
        assert_eq!(mp_iter.peek_n(1), None);
        assert_eq!(mp_iter.peek_n(1000), None);
    }
    #[test]
    fn test_peek_fold_full() {
        let inner: Vec<_> = (0..1000).collect();
        let mut mp_iter = MultiPeek::from(inner.clone().into_iter());
        assert_eq!(
            mp_iter.peek_fold(0, |a, b| Some(a + b)),
            inner.into_iter().sum::<i32>()
        );
        assert_eq!(mp_iter.next(), Some(0));
    }
    #[test]
    fn test_peek_fold_early_terminate() {
        let inner: Vec<_> = (0..1000).collect();
        let mut mp_iter = MultiPeek::from(inner.clone().into_iter());
        assert_eq!(
            mp_iter.peek_fold(0, |a, b| if *b < 500 { Some(a + b) } else { None }),
            inner.into_iter().filter(|&x| x < 500).sum::<i32>()
        );
        assert_eq!(mp_iter.next(), Some(0));
    }
}
