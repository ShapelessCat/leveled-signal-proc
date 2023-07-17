use std::marker::PhantomData;

use lsp_runtime::signal::SingnalProcessor;
use lsp_runtime::{UpdateContext, Timestamp, WithTimestamp};

pub struct LivenessChecker<F, E> {
    expiration_period: Timestamp,
    is_liveness_event: F,
    phantom: PhantomData<E>,
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

impl <F, E, I> SingnalProcessor<I> for LivenessChecker<F, E>
where
    I: Iterator<Item = E>,
    F: FnMut(&E) -> bool,
    E: WithTimestamp,
{
    type Input = Timestamp;

    type Output = bool;

    #[inline(always)]
    fn update(&mut self, ctx: &mut UpdateContext<I>, input: &Self::Input) -> Self::Output {
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
