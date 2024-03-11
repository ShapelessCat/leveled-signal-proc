use std::fmt::Debug;
use std::marker::PhantomData;

use serde::Serialize;

use lsp_runtime::signal::SignalProcessor;
use lsp_runtime::{Duration, Timestamp, UpdateContext, WithTimestamp};

/// This is the signal processor that analyzes the liveness of a session based on heartbeat signals.
/// The output constantly answering the question: Is current session still alive?
/// The liveness defined as we can find a heartbeat event within `expiration_period` amount of time.
/// Thus, this operator uses the look ahead mechanism of the LSP system to see if there's a future
/// heartbeat event.
#[derive(Serialize)]
pub struct LivenessChecker<IsLivenessEventFunc, Clock, Event> {
    is_liveness_event: IsLivenessEventFunc,
    expiration_period: Duration,
    last_event_clock: Clock,
    last_event_timestamp: Timestamp,
    phantom: PhantomData<Event>,
}

impl<F, C: Debug, E: Debug> Debug for LivenessChecker<F, C, E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LivenessChecker")
            .field("expiration_period", &self.expiration_period)
            .field("last_event_timestamp", &self.last_event_timestamp)
            .field("last_event_clock", &self.last_event_clock)
            .field("phantom", &self.phantom)
            .finish()
    }
}

impl<F, C: Default, E> LivenessChecker<F, C, E> {
    pub fn new(is_liveness_event: F, expiration_period: Duration) -> Self
    where
        F: FnMut(&E) -> bool,
    {
        Self {
            is_liveness_event,
            expiration_period,
            last_event_clock: Default::default(),
            last_event_timestamp: Default::default(),
            phantom: PhantomData,
        }
    }
}

impl<'a, I, F, C, E> SignalProcessor<'a, I> for LivenessChecker<F, C, E>
where
    I: Iterator<Item = E>,
    F: FnMut(&E) -> bool,
    C: Clone + PartialEq,
    E: WithTimestamp,
{
    type Input = C;

    type Output = bool;

    #[inline(always)]
    fn update(&mut self, ctx: &mut UpdateContext<I>, input: &'a Self::Input) -> Self::Output {
        if &self.last_event_clock != input {
            self.last_event_clock = input.clone();
            self.last_event_timestamp = ctx.frontier();
        }

        let look_ahead_cutoff = self.last_event_timestamp + self.expiration_period;

        ctx.peek_fold(false, |v, e| {
            if *v || e.timestamp() >= look_ahead_cutoff {
                return None;
            }
            Some((self.is_liveness_event)(e))
        })
    }
}

#[cfg(test)]
mod test {
    use lsp_runtime::signal::SignalProcessor;

    use crate::test::{create_lsp_context_for_test_from_input_slice, TestSignalInput};

    use super::LivenessChecker;

    #[test]
    fn test_liveness_checker() {
        let input = [
            1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1,
        ];
        let output = [
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 0,
        ];
        let mut context = create_lsp_context_for_test_from_input_slice(&input);
        let mut liveness = LivenessChecker::<_, _, TestSignalInput>::new(|data| data.value > 0, 6);
        let mut latch_output = 0;
        let mut buf = Default::default();

        let mut out_iter = output.iter();

        while let Some(m) = context.next_event(&mut buf) {
            if buf.value > 0 {
                latch_output = m.timestamp();
            }
            let value = if liveness.update(&mut context.borrow_update_context(), &latch_output) {
                1
            } else {
                0
            };
            assert_eq!(Some(value), out_iter.next().cloned())
        }
    }
}
