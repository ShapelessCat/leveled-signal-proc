use serde::Serialize;

use lsp_runtime::context::UpdateContext;
use lsp_runtime::signal_api::SignalMeasurement;
use lsp_runtime::{Duration, Timestamp};

#[derive(Clone, Default, Debug, Serialize)]
pub struct DurationSinceLastLevel<T> {
    last_assignment_timestamp: Timestamp,
    last_level: Option<T>,
}

impl<'a, I, T> SignalMeasurement<'a, I> for DurationSinceLastLevel<T>
where
    I: Iterator,
    T: Clone + Serialize,
{
    type Input = T;
    type Output = Duration;

    fn update(&mut self, ctx: &mut UpdateContext<I>, input: &'a Self::Input) {
        self.last_level = Some(input.clone());
        self.last_assignment_timestamp = ctx.frontier();
    }

    fn measure(&self, ctx: &mut UpdateContext<I>) -> Self::Output {
        if self.last_level.is_none() {
            0
        } else {
            ctx.frontier() - self.last_assignment_timestamp
        }
    }
}
