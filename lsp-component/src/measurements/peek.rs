use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use lsp_runtime::context::UpdateContext;
use lsp_runtime::signal_api::{Patchable, SignalMeasurement};
use lsp_runtime::Timestamp;

#[derive(Clone, Default, Debug, Serialize)]
pub struct Peek<T>(T);

impl<'a, I, T> SignalMeasurement<'a, I> for Peek<T>
where
    I: Iterator,
    T: Clone,
{
    type Input = T;

    type Output = T;

    fn update(&mut self, _: &mut UpdateContext<I>, v: &'a Self::Input) {
        self.0 = v.clone();
    }

    fn measure(&self, _: &mut UpdateContext<I>) -> Self::Output {
        self.0.clone()
    }
}

#[derive(Deserialize)]
pub struct PeekState<T>(T);

impl<T> Patchable for Peek<T>
where
    T: Serialize + DeserializeOwned,
{
    type State = PeekState<T>;

    // fn patch(&mut self, state: &str) {
    //     let state: Self::State = serde_json::from_str(state).unwrap();
    //     self.patch_from(state);
    // }

    fn patch_from(&mut self, state: Self::State) {
        self.0 = state.0;
    }
}

/// This is the measurement for timestamp.
/// Time is not a leveled signal, and we can't use the [Peek] measurement to measure time.
#[derive(Clone, Debug, Serialize)]
pub struct PeekTimestamp;

impl<'a, I: Iterator> SignalMeasurement<'a, I> for PeekTimestamp {
    type Input = Timestamp;

    type Output = u64;

    /// This method is designed for updating when a leveled signal changes. Since time is not a
    /// leveled signal, this method shouldn't do anything.
    fn update(&mut self, _: &mut UpdateContext<I>, _: &'a Self::Input) {}

    /// This method can't depend on any recorded value, because time keeps changing, and it is not a
    /// leveled signal.
    fn measure(&self, ctx: &mut UpdateContext<I>) -> Self::Output {
        ctx.frontier()
    }
}

#[derive(Deserialize)]
pub struct PeekTimestampState;

impl Patchable for PeekTimestamp {
    type State = PeekTimestampState;

    fn patch(&mut self, _state: &str) {}

    fn patch_from(&mut self, _state: Self::State) {}
}
