use std::marker::PhantomData;
use crate::{UpdateContext, Timestamp, WithTimestamp};

/// A computed leveled signal is a signal that is computed from other leveled signals.
pub trait ComputedLeveledSignal<EventIt: Iterator> {
    type Input;
    type Output;

    /// Update the signal - the data readiness contraint requires the output must be valid.
    /// The semantics of this method is follow: All the input signals are defined by parameter `input` from now. 
    /// And the output is also valid from the now.
    /// Data readiness isn't a problem in most of the computed signals. 
    fn update(&mut self, ctx: UpdateContext<EventIt>, input: &Self::Input) -> Self::Output;
}

/// Mapping input signals statelessly to a output signal
pub struct MappedSignal<T, U, F> {
    how: F,
    _phantom_data: PhantomData<(T, U)>,
}

impl <T, U, F> MappedSignal<T, U, F>
where
    F: FnMut(&T) -> U
{
    pub fn new(how: F) -> Self {
        MappedSignal { how, _phantom_data: PhantomData }
    }
}

impl <T, U, F, I:Iterator> ComputedLeveledSignal<I> for MappedSignal<T, U, F> 
where
    F: FnMut(&T) -> U
{
    type Input = T;

    type Output = U;

    #[inline(always)]
    fn update(&mut self, _: UpdateContext<I>, input: &T) -> U {
        (self.how)(input)
    }
}

#[derive(Default)]
pub struct Latch<T: Clone>(T);

impl <T: Clone, I:Iterator> ComputedLeveledSignal<I> for Latch<T> {
    type Input = (bool, T);
    type Output = T;
    #[inline(always)]
    fn update(&mut self, _: UpdateContext<I>, &(ref set, ref value): &Self::Input) -> T {
        if *set {
            self.0 = value.clone();
        }
        self.0.clone()
    }
}

pub struct LivenessSignal<F, E> {
    expiration_period: Timestamp,
    is_liveness_event: F,
    phantom: PhantomData<E>,
}

impl <F, E> LivenessSignal<F, E> {
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

impl <F, E, I> ComputedLeveledSignal<I> for LivenessSignal<F, E>
where
    I: Iterator<Item = E>,
    F: FnMut(&E) -> bool,
    E: WithTimestamp,
{
    type Input = Timestamp;

    type Output = bool;

    fn update(&mut self, mut ctx: UpdateContext<I>, input: &Self::Input) -> Self::Output {
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

#[derive(Default)]
pub struct ValueChangeCounter<T:Clone + Eq> {
    prev: Option<T>,
    counter: usize,
}

impl <T: Clone + Eq> ValueChangeCounter<T> {
    pub fn with_init_value(value: T) -> Self {
        Self {
            prev: Some(value),
            counter: 0,
        }
    }
}

impl <T: Clone + Eq, I: Iterator> ComputedLeveledSignal<I> for ValueChangeCounter<T> {
    type Input = T;

    type Output = usize;

    fn update(&mut self, _: UpdateContext<I>, input: &Self::Input) -> Self::Output {
        if self.prev.as_ref().map_or(true, |value| value != input) {
            self.counter += 1;
            self.prev = Some(input.clone());
        }
        self.counter
    }
}
