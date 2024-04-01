use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use lsp_runtime::context::UpdateContext;
use lsp_runtime::signal_api::{Patchable, SignalMeasurement};
use lsp_runtime::Timestamp;

/// Measure by peeking the input value.
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

#[cfg(test)]
mod test {
    use lsp_runtime::signal_api::{Patchable, SignalMeasurement};

    use super::{Peek, PeekTimestamp};
    use crate::test::create_lsp_context_for_test;

    #[test]
    fn test_peek() {
        let mut peek = Peek::default();
        let mut ctx = create_lsp_context_for_test();
        let mut uc = ctx.borrow_update_context();
        peek.update(&mut uc, &1);
        assert_eq!(peek.measure(&mut uc), 1);
        peek.update(&mut uc, &2);
        assert_eq!(peek.measure(&mut uc), 2);

        let state = peek.to_state();
        let mut init_peek = Peek::<i32>::default();
        init_peek.patch(&state);
        assert_eq!(state, init_peek.to_state());
    }

    #[test]
    fn test_peek_timestamp() {
        let mut peek_timestamp = PeekTimestamp;
        let mut ctx = create_lsp_context_for_test();
        let mut buf = Default::default();
        let mut out_iter = 0..100;
        while ctx.next_event(&mut buf).is_some() {
            peek_timestamp.update(&mut ctx.borrow_update_context(), &0);
            let value = peek_timestamp.measure(&mut ctx.borrow_update_context());
            assert_eq!(Some(value), out_iter.next().clone())
        }

        let state = peek_timestamp.to_state();
        let mut init_peek_timestamp = PeekTimestamp;
        init_peek_timestamp.patch(&state);
        assert_eq!(state, init_peek_timestamp.to_state());
    }
}
