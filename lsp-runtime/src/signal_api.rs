use serde::Serialize;

use crate::context::UpdateContext;

pub trait SignalProcessor<'a, EventIt: Iterator>: Serialize {
    type Input;
    type Output;

    /// Update the signal - the data readiness constraint requires the output must be valid.
    /// The semantics of this method: All the input signals are defined by the parameter `input`
    /// from now, and the output is also valid from the now.
    /// Data readiness isn't a problem in most of the computed signals.
    fn update(&mut self, ctx: &mut UpdateContext<EventIt>, input: &'a Self::Input) -> Self::Output;
}

/// A measurement is an inspection of the state of the signal processing system.
/// Although all the signal processor doesn't take timestamp as input, the measurement can be a
/// function of time.
/// For example, you can measure the duration since an output is true, etc.
pub trait SignalMeasurement<'a, EventIter: Iterator>: Serialize {
    type Input;
    type Output;

    /// Notify the value change take effect from now
    fn update(&mut self, ctx: &mut UpdateContext<EventIter>, input: &'a Self::Input);

    /// Measure the observation value now
    fn measure(&self, ctx: &mut UpdateContext<EventIter>) -> Self::Output;
}