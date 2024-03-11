use serde::Serialize;
use crate::UpdateContext;

pub trait SignalProcessor<'a, EventIt: Iterator>: Serialize {
    type Input;
    type Output;

    /// Update the signal - the data readiness constraint requires the output must be valid.
    /// The semantics of this method: All the input signals are defined by the parameter `input`
    /// from now, and the output is also valid from the now.
    /// Data readiness isn't a problem in most of the computed signals.
    fn update(&mut self, ctx: &mut UpdateContext<EventIt>, input: &'a Self::Input) -> Self::Output;
}
