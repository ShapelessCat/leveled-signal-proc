use crate::Timestamp;
use crate::UpdateContext;

/// A measurement is a inspection of the state of the signal processing system.
/// Although all the signal processor doesn't take timestamp as input, the measurement can be
/// a function of time. 
/// For example you can measure the duration since an output is true, etc.
pub trait Measurement<EventIter: Iterator> {
    type Input;
    type Output;

    #[inline(always)]
    fn reset(&mut self) {}

    // Notify the value change take effect from now
    fn update(&mut self, ctx: &mut UpdateContext<EventIter>, input: &Self::Input);

    fn measure_at(&self, ctx: &mut UpdateContext<EventIter>, timestamp: Timestamp) -> Self::Output;
}

#[derive(Default)]
pub struct Peek<T>(T);

impl <T : Clone, I: Iterator> Measurement<I> for Peek<T> {
    type Input = T;

    type Output = T;

    fn update(&mut self, _: &mut UpdateContext<I>, v: &Self::Input) {
        self.0 = v.clone();
    }

    fn measure_at(&self, _: &mut UpdateContext<I>, _: Timestamp) -> Self::Output {
        self.0.clone()
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