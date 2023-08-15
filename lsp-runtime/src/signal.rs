use crate::UpdateContext;

pub trait SignalProcessor<'a, EventIt: Iterator> {
    type Input: 'a;
    type Output;

    /// Update the signal - the data readiness contraint requires the output must be valid.
    /// The semantics of this method is follow: All the input signals are defined by parameter `input` from now.
    /// And the output is also valid from the now.
    /// Data readiness isn't a problem in most of the computed signals.
    fn update(&mut self, ctx: &mut UpdateContext<EventIt>, input: Self::Input) -> Self::Output;
}
