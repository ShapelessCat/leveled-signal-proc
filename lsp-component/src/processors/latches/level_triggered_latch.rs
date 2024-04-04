use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use lsp_runtime::signal_api::SignalProcessor;
use lsp_runtime::Duration;
use lsp_runtime::{context::UpdateContext, signal_api::Patchable};

use super::retention::{KeepForever, Retention, TimeToLive};

/// A level triggered latch is a signal processor that takes a control input and a data input.
/// For each time, a level triggered latch produces the same output as its internal state.
/// When the control input becomes true, the level triggered latch changes its internal state to the
/// data input. This concept borrowed from the hardware component which shares the same name. And
/// it's widely used as one bit memory in digital circuits.
#[derive(Default, Debug, Serialize)]
pub struct LevelTriggeredLatch<Data, RetentionPolicy: Retention<Data> = KeepForever> {
    data: Data,
    retention: RetentionPolicy,
}

impl<T: Clone> LevelTriggeredLatch<T> {
    pub fn with_initial_value(data: T) -> Self {
        Self {
            data,
            retention: KeepForever,
        }
    }
}

impl<T: Clone + Serialize + DeserializeOwned> LevelTriggeredLatch<T, TimeToLive<T>> {
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

impl<'a, I, T, R> SignalProcessor<'a, I> for LevelTriggeredLatch<T, R>
where
    I: Iterator,
    T: Clone,
    R: Retention<T>,
{
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

#[derive(Deserialize)]
pub struct LevelTriggeredLatchState<Data, RetentionPolicy> {
    data: Data,
    retention: RetentionPolicy,
}

impl<D, R: Retention<D>> Patchable for LevelTriggeredLatch<D, R>
where
    D: Serialize + DeserializeOwned,
    R: Serialize + DeserializeOwned,
{
    type State = LevelTriggeredLatchState<D, R>;

    fn patch_from(&mut self, state: Self::State) {
        self.data = state.data;
        self.retention = state.retention;
    }
}

#[cfg(test)]
mod test {
    use lsp_runtime::signal_api::SignalProcessor;

    use super::LevelTriggeredLatch;
    use crate::test::create_lsp_context_for_test;

    #[test]
    fn test_basic_latch() {
        let mut latch = LevelTriggeredLatch::with_initial_value(0);
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
        let mut latch = LevelTriggeredLatch::with_forget_behavior(0, 0, 2);
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
