use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use lsp_runtime::context::UpdateContext;
use lsp_runtime::signal_api::{Patchable, SignalProcessor};
use lsp_runtime::{Duration, Timestamp};

/// Note:
/// Although the duration of current level cannot be a measurement, as it's a function of time,
/// duration of previous level is a well-defined signal -- duration of previous level is a known
/// value.
#[derive(Default, Debug, Serialize)]
pub struct DurationOfPreviousLevel<Level> {
    current_value: Level,
    current_value_since: Timestamp,
    output_buf: Timestamp,
}

impl<'a, I, L> SignalProcessor<'a, I> for DurationOfPreviousLevel<L>
where
    I: Iterator,
    L: Clone + PartialEq,
{
    type Input = L;

    type Output = Duration;

    #[inline(always)]
    fn update(&mut self, ctx: &mut UpdateContext<I>, input: &'a Self::Input) -> Self::Output {
        if &self.current_value != input {
            self.output_buf = ctx.frontier() - self.current_value_since;
            self.current_value = input.clone();
            self.current_value_since = ctx.frontier();
        }
        self.output_buf
    }
}

#[derive(Deserialize)]
pub struct DurationOfPreviousLevelState<Level> {
    current_value: Level,
    current_value_since: Timestamp,
    output_buf: Timestamp,
}

impl<L> Patchable for DurationOfPreviousLevel<L>
where
    L: Serialize + DeserializeOwned,
{
    type State = DurationOfPreviousLevelState<L>;

    fn patch_from(&mut self, state: Self::State) {
        self.current_value = state.current_value;
        self.current_value_since = state.current_value_since;
        self.output_buf = state.output_buf;
    }
}

#[cfg(test)]
mod test {
    use lsp_runtime::signal_api::{Patchable, SignalProcessor};

    use crate::test::{create_lsp_context_for_test_from_input, TestSignalBag};

    use super::DurationOfPreviousLevel;

    #[test]
    fn test_duration_prev_level() {
        let mut duration_of_prev_level = DurationOfPreviousLevel::default();
        let input = [0, 0, 1, 1, 1, 0];
        let mut context = create_lsp_context_for_test_from_input(&input);
        let output = [0, 0, 2, 2, 2, 3];
        let mut output_iter = output.into_iter();
        let mut state_buf = TestSignalBag::default();

        while context.next_event(&mut state_buf).is_some() {
            let mut uc = context.borrow_update_context();
            let transformed_value = duration_of_prev_level.update(&mut uc, &state_buf.value);
            assert_eq!(transformed_value, output_iter.next().unwrap());
        }

        let state = duration_of_prev_level.to_state();
        let mut init_duration_of_prev_level = DurationOfPreviousLevel::<i32>::default();
        init_duration_of_prev_level.patch(&state);
        assert_eq!(state, init_duration_of_prev_level.to_state());
    }
}
