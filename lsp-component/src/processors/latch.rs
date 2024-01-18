use serde::{Deserialize, Serialize};

use lsp_runtime::signal::SignalProcessor;
use lsp_runtime::{Duration, Timestamp, UpdateContext};

/// Abstracts the retention behavior of a latch
pub trait Retention<T> {
    fn drop_timestamp(&mut self, now: Timestamp) -> Option<Timestamp>;
    fn should_drop(&mut self, now: Timestamp) -> Option<T>;
}

/// The retention policy for latches that keep the value forever
#[derive(Default, Debug, Deserialize, Serialize)]
pub struct KeepForever;

impl<T> Retention<T> for KeepForever {
    fn drop_timestamp(&mut self, _: Timestamp) -> Option<Timestamp> {
        None
    }

    fn should_drop(&mut self, _: Timestamp) -> Option<T> {
        None
    }
}

/// The retention policy for latches that keep the value for a period of time
#[derive(Debug, Deserialize, Serialize)]
pub struct TimeToLive<T> {
    default_value: T,
    value_forgotten_timestamp: Timestamp,
    time_to_live: Duration,
}

impl<T: Clone> Retention<T> for TimeToLive<T> {
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
/// For each time, a latch produces the same output as the internal state.
/// When the control input becomes true, the latch changes its internal state to the data input.
/// This concept borrowed from the hardware component which shares the same name. And it's widely
/// used as one bit memory in digital circuits.
#[derive(Default, Debug, Deserialize, Serialize)]
pub struct Latch<Data: Clone, RetentionPolicy: Retention<Data> = KeepForever> {
    data: Data,
    retention: RetentionPolicy,
}

#[derive(Default, Debug, Deserialize, Serialize)]
pub struct EdgeTriggeredLatch<Control, Data, RetentionPolicy: Retention<Data> = KeepForever> {
    last_control_level: Control,
    data: Data,
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

impl<C: Default, D> EdgeTriggeredLatch<C, D> {
    pub fn with_initial_value(data: D) -> Self {
        Self {
            last_control_level: Default::default(),
            data,
            retention: KeepForever,
        }
    }
}

impl<T: Clone> Latch<T, TimeToLive<T>> {
    pub fn with_forget_behavior(data: T, default: T, time_to_memorize: Duration) -> Self {
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

impl<C: Default, D: Clone> EdgeTriggeredLatch<C, D, TimeToLive<D>> {
    pub fn with_forget_behavior(data: D, default: D, time_to_memorize: Duration) -> Self {
        Self {
            data,
            last_control_level: Default::default(),
            retention: TimeToLive {
                default_value: default,
                value_forgotten_timestamp: 0,
                time_to_live: time_to_memorize,
            },
        }
    }
}

impl<'a, I: Iterator, T: Clone, R: Retention<T>> SignalProcessor<'a, I> for Latch<T, R> {
    type Input = (bool, T);
    type Output = T;
    #[inline(always)]
    fn update(&mut self, ctx: &mut UpdateContext<I>, (set, value): &'a Self::Input) -> T {
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

impl<'a, I: Iterator, C: PartialEq + Clone, D: Clone, R: Retention<D>> SignalProcessor<'a, I>
    for EdgeTriggeredLatch<C, D, R>
{
    type Input = (C, D);
    type Output = D;
    #[inline(always)]
    fn update(&mut self, ctx: &mut UpdateContext<I>, (control, value): &'a Self::Input) -> D {
        let should_set = if &self.last_control_level != control {
            self.last_control_level = control.clone();
            true
        } else {
            false
        };
        if should_set {
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

#[cfg(test)]
mod test {
    use lsp_runtime::signal::SignalProcessor;

    use crate::{processors::Latch, test::create_lsp_context_for_test};

    #[test]
    fn test_basic_latch() {
        let mut latch = Latch::with_initial_value(0);
        let mut c = create_lsp_context_for_test();
        let mut ctx = c.borrow_update_context();
        assert_eq!(latch.update(&mut ctx, &(true, 1)), 1);
        assert_eq!(latch.update(&mut ctx, &(false, 2)), 1);
        assert_eq!(latch.update(&mut ctx, &(true, 3)), 3);
        assert_eq!(latch.update(&mut ctx, &(false, 4)), 3);
        assert_eq!(latch.update(&mut ctx, &(false, 5)), 3);
        assert_eq!(latch.update(&mut ctx, &(true, 6)), 6);
        assert_eq!(latch.update(&mut ctx, &(false, 7)), 6);
    }

    #[test]
    fn test_forget_behavior() {
        let mut latch = Latch::with_forget_behavior(0, 0, 2);
        let mut c = create_lsp_context_for_test();
        let mut buf = Default::default();

        c.next_event(&mut buf).unwrap();
        let mut ctx = c.borrow_update_context();
        assert_eq!(latch.update(&mut ctx, &(true, 1)), 1);

        c.next_event(&mut buf).unwrap();
        let mut ctx = c.borrow_update_context();
        assert_eq!(latch.update(&mut ctx, &(false, 2)), 1);

        c.next_event(&mut buf).unwrap();
        let mut ctx = c.borrow_update_context();
        assert_eq!(latch.update(&mut ctx, &(false, 2)), 0);

        c.next_event(&mut buf).unwrap();
        let mut ctx = c.borrow_update_context();
        assert_eq!(latch.update(&mut ctx, &(true, 2)), 2);

        c.next_event(&mut buf).unwrap();
        let mut ctx = c.borrow_update_context();
        assert_eq!(latch.update(&mut ctx, &(true, 2)), 2);

        c.next_event(&mut buf).unwrap();
        let mut ctx = c.borrow_update_context();
        assert_eq!(latch.update(&mut ctx, &(false, 3)), 2);

        c.next_event(&mut buf).unwrap();
        let mut ctx = c.borrow_update_context();
        assert_eq!(latch.update(&mut ctx, &(false, 2)), 0);
        c.next_event(&mut buf).unwrap();
        let mut ctx = c.borrow_update_context();
        assert_eq!(latch.update(&mut ctx, &(false, 2)), 0);
    }
}
