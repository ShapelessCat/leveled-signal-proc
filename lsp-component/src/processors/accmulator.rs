use std::ops::AddAssign;

use lsp_runtime::{signal::SignalProcessor, UpdateContext};

/// An accumlator is a signal processor that constantly add input to the internal state.
/// Normally accumulator doesn't add input to the internal state, until it sees the control signal
/// has changed.
#[derive(Debug)]
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

    #[inline(always)]
    fn update(&mut self, _: &mut UpdateContext<I>, (control, data): Self::Input) -> Self::Output {
        if &self.prev_control_signal != control {
            if (self.filter)(control) {
                self.accumulator += data.clone();
            }
            self.prev_control_signal = control.clone();
        }
        self.accumulator.clone()
    }
}

#[cfg(test)]
mod test {
    use super::Accumulator;
    use crate::test::create_lsp_context_for_test;
    use lsp_runtime::signal::SignalProcessor;

    #[test]
    fn test_basic_logic() {
        let mut counter = Accumulator::with_event_filter(0, |_| true);
        let mut ctx = create_lsp_context_for_test();
        let mut uc = ctx.borrow_update_context();

        assert_eq!(0, counter.update(&mut uc, (&0, &1)));
        assert_eq!(1, counter.update(&mut uc, (&1, &1)));
        assert_eq!(1, counter.update(&mut uc, (&1, &2)));
        assert_eq!(4, counter.update(&mut uc, (&2, &3)));
    }

    #[test]
    fn test_signal_filter() {
        let mut counter = Accumulator::with_event_filter(0, |&x| x % 2 == 0);
        let mut ctx = create_lsp_context_for_test();
        let mut uc = ctx.borrow_update_context();

        assert_eq!(0, counter.update(&mut uc, (&0, &1)));
        assert_eq!(0, counter.update(&mut uc, (&1, &1)));
        assert_eq!(2, counter.update(&mut uc, (&2, &2)));
        assert_eq!(2, counter.update(&mut uc, (&3, &3)));
        assert_eq!(6, counter.update(&mut uc, (&4, &4)));
    }
}
