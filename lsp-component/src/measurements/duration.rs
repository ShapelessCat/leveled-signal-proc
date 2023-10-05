use lsp_runtime::{measurement::Measurement, Timestamp, UpdateContext};

use super::combinator::ScopedMeasurement;

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

pub type ScopedDurationTrue<T> = ScopedMeasurement<T, DurationTrue, Timestamp>;

/* 
#[derive(Default)]
pub struct ScopedDurationTrue<T: Clone> {
    current_control_level: T,
    current_level_timestamp: Timestamp,
    inner: DurationTrue,
    current_base: Timestamp,
    prev_base: Option<Timestamp>,
}

impl<'a, T:Clone + Eq + 'a, I: Iterator> Measurement<'a, I> for ScopedDurationTrue<T> {
    type Input = (&'a T, &'a bool);
    type Output = Timestamp;

    fn update(&mut self, ctx: &mut UpdateContext<I>, (level, data): (&T, &bool)) {
        self.inner.update(ctx, data);

        if &self.current_control_level != level {
            self.current_control_level = level.clone();
            self.current_level_timestamp = ctx.frontier();
            self.prev_base = Some(self.current_base);
            self.current_base = self.inner.measure(ctx);
        }
    }

    fn measure(&self, ctx: &mut UpdateContext<I>) -> Self::Output {
        let base = match self.prev_base {
            Some(prev_base) if prev_base == ctx.frontier() => prev_base,
            _ => self.current_base,
        };
        self.inner.measure(ctx) - base
    }
}*/