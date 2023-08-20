use lsp_runtime::{measurement::Measurement, Timestamp, UpdateContext};

#[derive(Default)]
pub struct DurationSinceBecomeTrue {
    last_input: bool,
    last_assignment_timestamp: Timestamp,
}

impl<'a, I: Iterator> Measurement<'a, I> for DurationSinceBecomeTrue {
    type Input = &'a bool;
    type Output = Timestamp;

    fn update(&mut self, ctx: &mut UpdateContext<I>, input: Self::Input) {
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

#[derive(Default)]
pub struct DurationTrue {
    last_input: bool,
    last_input_timestamp: Timestamp,
    cur_input: bool,
    cur_input_timestamp: Timestamp,
    accumulated_duration: Timestamp,
}

impl<'a, I: Iterator> Measurement<'a, I> for DurationTrue {
    type Input = &'a bool;
    type Output = Timestamp;

    fn update(&mut self, ctx: &mut UpdateContext<I>, input: &bool) {
        if self.last_input {
            self.accumulated_duration += self.cur_input_timestamp - self.last_input_timestamp;
        }
        self.last_input = self.cur_input;
        self.last_input_timestamp = self.cur_input_timestamp;
        self.cur_input = *input;
        self.cur_input_timestamp = ctx.frontier();
    }

    fn measure(&self, ctx: &mut UpdateContext<I>) -> Self::Output {
        let timestamp = ctx.frontier();
        //assert!(self.last_input_timestamp <= timestamp && timestamp <= self.cur_input_timestamp);
        self.accumulated_duration
            + if self.last_input {
                timestamp - self.last_input_timestamp
            } else {
                0
            }
    }
}
