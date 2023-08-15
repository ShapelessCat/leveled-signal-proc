use crate::UpdateContext;

/// A measurement is a inspection of the state of the signal processing system.
/// Although all the signal processor doesn't take timestamp as input, the measurement can be
/// a function of time.
/// For example you can measure the duration since an output is true, etc.
pub trait Measurement<'a, EventIter: Iterator> {
    type Input: 'a;
    type Output;

    #[inline(always)]
    fn reset(&mut self) {}

    /// Notify the value change take effect from now
    fn update(&mut self, ctx: &mut UpdateContext<EventIter>, input: Self::Input);

    /// Measure the observation value now
    fn measure(&self, ctx: &mut UpdateContext<EventIter>) -> Self::Output;
}
