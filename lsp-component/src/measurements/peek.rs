use lsp_runtime::{measurement::Measurement, Timestamp, UpdateContext};

use super::combinator::ScopedMeasurement;

#[derive(Default, Debug)]
pub struct Peek<T>(T);

impl<'a, T: Clone + 'a, I: Iterator> Measurement<'a, I> for Peek<T> {
    type Input = &'a T;

    type Output = T;

    fn update(&mut self, _: &mut UpdateContext<I>, v: Self::Input) {
        self.0 = v.clone();
    }

    fn measure(&self, _: &mut UpdateContext<I>) -> Self::Output {
        self.0.clone()
    }
}

pub type DiffSinceCurrentLevel<C, T> = ScopedMeasurement<C, Peek<T>, T>;

#[derive(Default, Debug)]
pub struct PeekTimestamp(Timestamp);

impl<'a, I: Iterator> Measurement<'a, I> for PeekTimestamp {
    type Input = &'a Timestamp;

    type Output = u64;

    fn update(&mut self, ctx: &mut UpdateContext<I>, _: Self::Input) {
        self.0 = ctx.frontier();
    }

    fn measure(&self, _: &mut UpdateContext<I>) -> Self::Output {
        self.0
    }
}
