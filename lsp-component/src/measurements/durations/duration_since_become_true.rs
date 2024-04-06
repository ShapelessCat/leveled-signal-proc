use serde::{Deserialize, Serialize};

use lsp_runtime::context::UpdateContext;
use lsp_runtime::signal_api::{Patchable, SignalMeasurement};
use lsp_runtime::{Duration, Timestamp};

#[derive(Clone, Default, Serialize)]
pub struct DurationSinceBecomeTrue {
    last_input: bool,
    last_assignment_timestamp: Timestamp,
}

impl<'a, I: Iterator> SignalMeasurement<'a, I> for DurationSinceBecomeTrue {
    type Input = bool;
    type Output = Duration;

    fn update(&mut self, ctx: &mut UpdateContext<I>, input: &'a Self::Input) {
        if *input != self.last_input {
            self.last_input = *input;
            self.last_assignment_timestamp = ctx.frontier();
        }
    }

    fn measure(&self, ctx: &mut UpdateContext<I>) -> Self::Output {
        if self.last_input {
            ctx.frontier() - self.last_assignment_timestamp
        } else {
            0
        }
    }
}

#[derive(Deserialize)]
pub struct DurationSinceBecomeTrueState {
    last_input: bool,
    last_assignment_timestamp: Timestamp,
}

impl Patchable for DurationSinceBecomeTrue {
    type State = DurationSinceBecomeTrueState;

    fn patch_from(&mut self, state: Self::State) {
        self.last_input = state.last_input;
        self.last_assignment_timestamp = state.last_assignment_timestamp;
    }
}
