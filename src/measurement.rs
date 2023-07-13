use crate::UpdateQueue;
use crate::Timestamp;

/// A measurement is a inspection of the state of the signal processing system.
/// Although all the signal processor doesn't take timestamp as input, the measurement can be
/// a function of time. 
/// For example you can measure the duration since an output is true, etc.
pub trait Measurement {
    type Input;
    type Output;

    #[inline(always)]
    fn reset(&mut self) {}

    /// This method reports the last output, the output will be valid til time stamp ctx.now()
    /// And after this update is called the measure_when method can be called to take measurement
    /// before now
    fn update(&mut self, ctx: &mut UpdateQueue, end_ts: Timestamp, input: &Self::Input);

    fn measure_at(&self, timestamp: Timestamp) -> Self::Output;
}

#[derive(Default)]
pub struct Peek<T>(T);

impl <T : Clone> Measurement for Peek<T> {
    type Input = T;

    type Output = T;

    fn update(&mut self, _: &mut UpdateQueue, _: Timestamp, v: &Self::Input) {
        self.0 = v.clone();
    }

    fn measure_at(&self, _: Timestamp) -> Self::Output {
        self.0.clone()
    }
}

#[derive(Default)]
pub struct Duration {
    last_value: bool,
    begin_timestamp: Timestamp,
    end_timestamp: Timestamp,
}

impl Measurement for Duration {
    type Input = bool;

    type Output = u64;

    fn update(&mut self, _: &mut UpdateQueue, end_ts: Timestamp, input: &Self::Input) {
        self.last_value = *input;
        self.begin_timestamp = self.end_timestamp;
        self.end_timestamp = end_ts;
    }

    fn measure_at(&self, timestamp: Timestamp) -> Self::Output {
        if self.last_value {
            timestamp.saturating_sub(self.begin_timestamp)
        } else {
            0
        }
    }
}

#[derive(Default)]
pub struct DurationTrue {
    last_value: bool,
    begin_timestamp: Timestamp,
    end_timestamp: Timestamp,
    accumulator: Timestamp,
}

impl Measurement for DurationTrue {
    type Input = bool;

    type Output = u64;

    fn update(&mut self, _: &mut UpdateQueue, end_ts: Timestamp, input: &Self::Input) {
        if self.last_value {
            self.accumulator += self.end_timestamp - self.begin_timestamp;
        }
        self.last_value = *input;
        self.begin_timestamp = self.end_timestamp;
        self.end_timestamp = end_ts;
    }

    fn measure_at(&self, timestamp: Timestamp) -> Self::Output {
        self.accumulator + if self.last_value {
            timestamp - self.begin_timestamp
        } else {
            0
        }
    }
}
