use std::marker::PhantomData;

use lsp_runtime::signal::SignalProcessor;
use lsp_runtime::{UpdateContext, Timestamp, WithTimestamp};

/// This is the signal processor that analyze the liveness of a session based on heartbeat signal.
/// The output constantly answering the question: Is current session still alive?
/// The liveness defined as we can find a heartbeat event within `expiuration_period` amount of time. 
/// Thus, this operator uses the look ahead mechamism of the LSP system to see if there's a future heartbeat event.
pub struct LivenessChecker<IsLivenessEventFunc, Event> {
    expiration_period: Timestamp,
    is_liveness_event: IsLivenessEventFunc,
    phantom: PhantomData<Event>,
}

impl <F, E> LivenessChecker<F, E> {
    pub fn new(is_liveness_event: F, time_window: Timestamp) -> Self 
    where
        F: FnMut(&E) -> bool
    {
        Self {
            expiration_period: time_window,
            is_liveness_event,
            phantom: PhantomData,
        }
    }
}

impl <'a, F, E, I> SignalProcessor<'a, I> for LivenessChecker<F, E>
where
    I: Iterator<Item = E>,
    F: FnMut(&E) -> bool,
    E: WithTimestamp,
{
    type Input = &'a Timestamp;

    type Output = bool;

    #[inline(always)]
    fn update(&mut self, ctx: &mut UpdateContext<I>, input: Self::Input) -> Self::Output {
        let look_ahead_cutoff = input + self.expiration_period;
        let output = ctx.peek_fold(false, |v, e| {
            if *v || e.timestamp() >= look_ahead_cutoff {
                return None;
            }
            Some((self.is_liveness_event)(e))
        });
        output
    }
}
