use serde::{Deserialize, Serialize};

use lsp_runtime::{measurement::Measurement, Timestamp, UpdateContext};

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
pub struct LinearChange {
    current_rate: f64,
    current_rate_start: Timestamp,
    accumulated_amount: f64,
}

impl<'a, I: Iterator> Measurement<'a, I> for LinearChange {
    type Input = &'a f64;
    type Output = f64;

    fn update(&mut self, ctx: &mut UpdateContext<I>, input: Self::Input) {
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
