use lsp_runtime::signal::SignalProcessor;
use lsp_runtime::{Timestamp, UpdateContext};

/// Abstracts the retention behavior of a latch
pub trait Rentention<T> {
    fn drop_timestamp(&mut self, now: Timestamp) -> Option<Timestamp>;
    fn should_drop(&mut self, now: Timestamp) -> Option<T>;
}

/// The retention policy for latch that keeps the value forever
#[derive(Default)]
pub struct KeepForever;

impl<T> Rentention<T> for KeepForever {
    fn drop_timestamp(&mut self, _: Timestamp) -> Option<Timestamp> {
        None
    }

    fn should_drop(&mut self, _: Timestamp) -> Option<T> {
        None
    }
}

/// The retention policy for latch that keeps the value for a period of time
pub struct TimeToLive<T> {
    default_value: T,
    value_forgotten_timestamp: Timestamp,
    time_to_live: Timestamp,
}

impl<T: Clone> Rentention<T> for TimeToLive<T> {
    fn drop_timestamp(&mut self, now: Timestamp) -> Option<Timestamp> {
        self.value_forgotten_timestamp = now + self.time_to_live;
        Some(self.time_to_live)
    }

    fn should_drop(&mut self, now: Timestamp) -> Option<T> {
        if self.value_forgotten_timestamp <= now {
            Some(self.default_value.clone())
        } else {
            None
        }
    }
}

/// A latch is a signal processor that takes a control input and a data input.
/// For each time, a latch produce the same output as the internal state.
/// When the control input becomes true, the latch change it internal state to the data input.
/// This concept borrowed from the hardware component which shares the same name. And it's widely use
/// as one bit memory in digital circuits.
#[derive(Default)]
pub struct Latch<DataType: Clone, RetentionPolicy: Rentention<DataType> = KeepForever> {
    data: DataType,
    retention: RetentionPolicy,
}

impl<T: Clone> Latch<T> {
    pub fn with_initial_value(data: T) -> Self {
        Self {
            data,
            retention: KeepForever,
        }
    }
}

impl<T: Clone> Latch<T, TimeToLive<T>> {
    pub fn with_forget_behavior(data: T, default: T, time_to_memorize: Timestamp) -> Self {
        Self {
            data,
            retention: TimeToLive {
                default_value: default,
                value_forgotten_timestamp: 0,
                time_to_live: time_to_memorize,
            },
        }
    }
}

impl<'a, T: Clone + 'a, I: Iterator, R: Rentention<T>> SignalProcessor<'a, I> for Latch<T, R> {
    type Input = (&'a bool, &'a T);
    type Output = T;
    #[inline(always)]
    fn update(&mut self, ctx: &mut UpdateContext<I>, (set, value): Self::Input) -> T {
        if *set {
            self.data = value.clone();
            if let Some(ttl) = self.retention.drop_timestamp(ctx.frontier()) {
                ctx.schedule_signal_update(ttl);
            }
        } else if let Some(value) = self.retention.should_drop(ctx.frontier()) {
            self.data = value;
        }
        self.data.clone()
    }
}
