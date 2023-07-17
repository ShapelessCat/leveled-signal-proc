use std::ops::AddAssign;

use lsp_runtime::signal::SingnalProcessor;

/// An accumlator is a signal processor that constantly add input to the internal state.
/// Normally accumulator doesn't add input to the internal state, until it see the control signal
/// has changed.
pub struct Accumulator<T, C, F> {
    prev_control_signal: C,
    filter: F,
    accumulator: T,
}

impl <T, C, F> Accumulator<T, C, F> 
where
    T: AddAssign<T> + Clone,
    C: Clone + PartialEq + Default, 
    F: Fn(&C) -> bool,
{
    pub fn new(init_value: T, filter: F) -> Self {
        Self {
            accumulator: init_value,
            prev_control_signal: C::default(),
            filter,
        }
    }
}

impl <T, C, I, F> SingnalProcessor<I> for Accumulator<T, C, F>
where
    I: Iterator,
    T: AddAssign<T> + Clone,
    C: Clone + PartialEq,
    F: Fn(&C) -> bool,
{
    type Input = (C, T);

    type Output = T;

    fn update(&mut self, _ctx: &mut lsp_runtime::UpdateContext<I>, &(ref this_signal, ref accu_input): &Self::Input) -> Self::Output {
        if &self.prev_control_signal != this_signal {
            if (self.filter)(this_signal) {
                self.accumulator += accu_input.clone();
            }
            self.prev_control_signal = this_signal.clone();
        }
        self.accumulator.clone()
    }
}