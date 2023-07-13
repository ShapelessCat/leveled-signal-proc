use std::marker::PhantomData;
use crate::UpdateQueue;

/// A leveled Signal is a signal that isn't constantly changing overtime
/// Which means the signal is only a function of input rather than a function of time.
/// A leveled signal may or may not be stateful - it can relies on the previous input, 
/// The only limit at this point is it doesn't take timestamp as input.
/// 
/// At this point the rule for the output validity described as: 
/// data is valid after flush method is called
/// 
/// The psuedocode for signal update:
/// ```
///     let input = get_some_input();
///     flush_all_leveled_signals();
///     if should_measure {
///         take_measurements();
///     }
///     update_all_leveld_signals();
/// ```
/// 
/// Consider a signal that indicates a session liveness - a session is dead when no more event in 90 seconds. 
/// Thus once we get a event, we don't know if the session liveness signal's value until we have either 90 seconds expiration or we get more heartbeat at this point.
/// So that we don't know the value of the liveness until we called the flush function, since we then know how long the leveled value last.
/// 
/// normally, update is just for resetting state of a stateful value / memorize the previous input. But it definitely can be more complex than that.
pub trait LeveledSignal {
    type Input;
    type Output;

    /// This function is called when the input is seen and the global epoch is 
    /// at the time that this input is becoming effective.
    /// 
    /// For a stateful signal processor, this is the time to reset the state if needed.
    /// The reset signal isn't a signal anymore, since it's actually a input.
    fn update(&mut self, ctx: &mut UpdateQueue, input: &Self::Input, output: &mut Self::Output);

    /// This function is called when next input is is seen and the global epoch is at the 
    /// time that the next input is effective. At this point, we actually know how long this
    /// leveled value last. 
    fn flush(&mut self, ctx: &mut UpdateQueue, output: &mut Self::Output);
}

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

impl <T, U, F> LeveledSignal for MappedSignal<T, U, F> 
where
    F: FnMut(&T) -> U
{
    type Input = T;

    type Output = U;

    #[inline(always)]
    fn update(&mut self, _: &mut UpdateQueue, input: &T, output: &mut U) {
        *output = (self.how)(input);
    }

    #[inline(always)]
    fn flush(&mut self, _: &mut UpdateQueue, _: &mut U) {}
}