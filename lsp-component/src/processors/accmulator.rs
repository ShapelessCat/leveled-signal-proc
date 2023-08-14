use std::ops::AddAssign;

use lsp_runtime::{signal::SignalProcessor, UpdateContext};

/// An accumlator is a signal processor that constantly add input to the internal state.
/// Normally accumulator doesn't add input to the internal state, until it see the control signal
/// has changed.
pub struct Accumulator<Data, ControlSignal, Filter> {
    prev_control_signal: ControlSignal,
    filter: Filter,
    accumulator: Data,
}

impl<T, C, F> Accumulator<T, C, F>
where
    T: AddAssign<T> + Clone,
    C: Clone + PartialEq + Default,
    F: Fn(&C) -> bool,
{
    pub fn with_event_filter(init_value: T, filter: F) -> Self {
        Self {
            accumulator: init_value,
            prev_control_signal: C::default(),
            filter,
        }
    }
}

impl<'a, T, C, I, F> SignalProcessor<'a, I> for Accumulator<T, C, F>
where
    I: Iterator,
    T: AddAssign<T> + Clone + 'a,
    C: Clone + PartialEq + 'a,
    F: Fn(&C) -> bool,
{
    type Input = (&'a C, &'a T);

    type Output = T;

    fn update(
        &mut self,
        _: &mut UpdateContext<I>,
        (control, data): Self::Input,
    ) -> Self::Output {
        if &self.prev_control_signal != control {
            if (self.filter)(control) {
                self.accumulator += data.clone();
            }
            self.prev_control_signal = control.clone();
        }
        self.accumulator.clone()
    }
}
