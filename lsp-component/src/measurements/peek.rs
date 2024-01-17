use serde::{Deserialize, Serialize};

use lsp_runtime::{measurement::Measurement, Timestamp, UpdateContext};

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
pub struct Peek<T>(T);

impl<'a, I: Iterator, T: Clone> Measurement<'a, I> for Peek<T> {
    type Input = T;

    type Output = T;

    fn update(&mut self, _: &mut UpdateContext<I>, v: &'a Self::Input) {
        self.0 = v.clone();
    }

    fn measure(&self, _: &mut UpdateContext<I>) -> Self::Output {
        self.0.clone()
    }
}

/// This is the measurement for timestamp.
/// Time is not a leveled signal, and we can't use the [Peek] measurement to measure time.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PeekTimestamp;

impl<'a, I: Iterator> Measurement<'a, I> for PeekTimestamp {
    type Input = Timestamp;

    type Output = u64;

    /// This method is designed for updating when a leveled signal changes. Since time is not a
    /// leveled signal, this method shouldn't do anything.
    fn update(&mut self, _: &mut UpdateContext<I>, _: &'a Self::Input) {}

    /// This method can't depend on any recorded value, because time keeps changing and it is not a
    /// leveled signal.
    fn measure(&self, ctx: &mut UpdateContext<I>) -> Self::Output {
        ctx.frontier()
    }
}
