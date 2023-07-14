use std::marker::PhantomData;
use crate::UpdateContext;

/// A computed leveled signal is a signal that is computed from other leveled signals.
pub trait ComputedLeveledSignal {
    type Input;
    type Output;

    /// Update the signal - the data readiness contraint requires the output must be valid.
    /// The semantics of this method is follow: All the input signals are defined by parameter `input` from now. 
    /// And the output is also valid from the now.
    /// Data readiness isn't a problem in most of the computed signals. 
    fn update<I:Iterator>(&mut self, ctx: UpdateContext<I>, input: &Self::Input) -> Self::Output;
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

impl <T, U, F> ComputedLeveledSignal for MappedSignal<T, U, F> 
where
    F: FnMut(&T) -> U
{
    type Input = T;

    type Output = U;

    #[inline(always)]
    fn update<I:Iterator>(&mut self, _: UpdateContext<I>, input: &T) -> U {
        (self.how)(input)
    }
}