use serde::{Deserialize, Serialize};

use lsp_runtime::{Duration, measurement::Measurement, Timestamp, UpdateContext};

#[derive(Clone, Default, Deserialize, Serialize)]
pub struct DurationSinceBecomeTrue {
    last_input: bool,
    last_assignment_timestamp: Timestamp,
}

impl<'a, I: Iterator> Measurement<'a, I> for DurationSinceBecomeTrue {
    type Input = &'a bool;
    type Output = Duration;

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

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
pub struct DurationSinceLastLevel<T: Clone> {
    last_assignment_timestamp: Timestamp,
    last_level: Option<T>,
}

impl<'a, I: Iterator, T: Clone + 'a> Measurement<'a, I> for DurationSinceLastLevel<T> {
    type Input = &'a T;
    type Output = Duration;

    fn update(&mut self, ctx: &mut UpdateContext<I>, input: Self::Input) {
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

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
pub struct DurationTrue {
    current_state: bool,
    accumulated_duration: Duration,
    last_true_starts: Timestamp,
}

impl<'a, I: Iterator> Measurement<'a, I> for DurationTrue {
    type Input = &'a bool;
    type Output = Duration;

    fn update(&mut self, ctx: &mut UpdateContext<I>, &input: &bool) {
        match (self.current_state, input) {
            (false, true) => {
                self.last_true_starts = ctx.frontier();
            }
            (true, false) => {
                self.accumulated_duration += ctx.frontier() - self.last_true_starts;
            }
            _ => (),
        };
        self.current_state = input;
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
    type Output = Duration;

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
