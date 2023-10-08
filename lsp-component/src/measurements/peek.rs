use lsp_runtime::{measurement::Measurement, UpdateContext};

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