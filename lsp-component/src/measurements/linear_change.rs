use serde::Serialize;

use lsp_runtime::context::UpdateContext;
use lsp_runtime::signal_api::SignalMeasurement;
use lsp_runtime::Timestamp;

#[derive(Clone, Default, Debug, Serialize)]
pub struct LinearChange {
    current_rate: f64,
    current_rate_start: Timestamp,
    accumulated_amount: f64,
}

impl<'a, I: Iterator> SignalMeasurement<'a, I> for LinearChange {
    type Input = f64;
    type Output = f64;

    fn update(&mut self, ctx: &mut UpdateContext<I>, input: &'a Self::Input) {
        if self.current_rate != *input {
            let duration = ctx.frontier() - self.current_rate_start;
            self.accumulated_amount += self.current_rate * duration as f64;
            self.current_rate = *input;
            self.current_rate_start = ctx.frontier();
        }
    }

    fn measure(&self, ctx: &mut UpdateContext<I>) -> Self::Output {
        let duration = ctx.frontier() - self.current_rate_start;
        let current_level_change = self.current_rate * duration as f64;
        (self.accumulated_amount + current_level_change) / 1e9
    }
}
