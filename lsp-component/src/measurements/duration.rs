use lsp_runtime::{Timestamp, measurement::Measurement, UpdateContext};

#[derive(Default)]
pub struct DurationSinceBecomeTrue {
    last_input: bool,
    last_assignment_timestamp: Timestamp,    
}

impl <I:Iterator> Measurement<I> for DurationSinceBecomeTrue {
    type Input = bool;
    type Output = Timestamp;

    fn update(&mut self, ctx: &mut UpdateContext<I>, input: &Self::Input) {
        if *input  != self.last_input {
            self.last_input = *input;
            self.last_assignment_timestamp = ctx.frontier();
        }
    }

    fn measure_at(&self, _: &mut UpdateContext<I>, timestamp: Timestamp) -> Self::Output {
        if self.last_input {
            timestamp - self.last_assignment_timestamp
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

impl <I:Iterator> Measurement<I> for DurationTrue {
    type Input = bool;
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

    fn measure_at(&self, _: &mut UpdateContext<I>, timestamp: Timestamp) -> Self::Output {
        assert!(self.last_input_timestamp <= timestamp && timestamp <= self.cur_input_timestamp);
        self.accumulated_duration + if self.last_input {
            timestamp - self.last_input_timestamp    
        } else {
            0
        }
    }
}