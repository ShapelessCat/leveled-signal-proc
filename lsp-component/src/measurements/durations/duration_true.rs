use lsp_runtime::context::UpdateContext;
use lsp_runtime::signal_api::{Patchable, SignalMeasurement};
use lsp_runtime::{Duration, Timestamp};
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, Serialize)]
pub struct DurationTrue {
    current_state: bool,
    accumulated_duration: Duration,
    last_true_starts: Timestamp,
}

impl<'a, I: Iterator> SignalMeasurement<'a, I> for DurationTrue {
    type Input = bool;
    type Output = Duration;

    fn update(&mut self, ctx: &mut UpdateContext<I>, input: &'a Self::Input) {
        match (self.current_state, input) {
            (false, true) => {
                self.last_true_starts = ctx.frontier();
            }
            (true, false) => {
                self.accumulated_duration += ctx.frontier() - self.last_true_starts;
            }
            _ => (),
        };
        self.current_state = *input;
    }

    fn measure(&self, ctx: &mut UpdateContext<I>) -> Self::Output {
        let timestamp = ctx.frontier();

        let current_state_duration = if self.current_state {
            timestamp - self.last_true_starts
        } else {
            0
        };

        self.accumulated_duration + current_state_duration
    }
}

#[derive(Deserialize)]
pub struct DurationTrueState {
    current_state: bool,
    accumulated_duration: Duration,
    last_true_starts: Timestamp,
}

impl Patchable for DurationTrue {
    type State = DurationTrueState;

    fn patch_from(&mut self, state: Self::State) {
        self.current_state = state.current_state;
        self.accumulated_duration = state.accumulated_duration;
        self.last_true_starts = state.last_true_starts;
    }
}
