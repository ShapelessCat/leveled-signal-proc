use lsp_runtime::context::UpdateContext;
use lsp_runtime::signal_api::{Patchable, SignalMeasurement};
use lsp_runtime::{Duration, Timestamp};
use serde::Serialize;

/// Measure the duration during which the input signal is `true`, yielding a cumulative result
/// accounting for all `true` levels up to the current measurement time.
#[derive(Clone, Default, Debug, Serialize, Patchable)]
pub struct DurationTrue {
    current_state: bool,
    accumulated_duration: Duration,
    last_true_starts: Timestamp,
}

impl<'a, I: Iterator> SignalMeasurement<'a, I> for DurationTrue {
    type Input = bool;
    type Output = Duration;

    fn update(&mut self, ctx: &mut UpdateContext<I>, input: &'a Self::Input) {
        match (self.current_state, input) {
            (false, true) => {
                self.last_true_starts = ctx.frontier();
            }
            (true, false) => {
                self.accumulated_duration += ctx.frontier() - self.last_true_starts;
            }
            _ => (),
        };
        self.current_state = *input;
    }

    fn measure(&self, ctx: &mut UpdateContext<I>) -> Self::Output {
        let timestamp = ctx.frontier();

        let current_state_duration = if self.current_state {
            timestamp - self.last_true_starts
        } else {
            0
        };

        self.accumulated_duration + current_state_duration
    }
}

// #[derive(Deserialize)]
// pub struct DurationTrueState {
//     current_state: bool,
//     accumulated_duration: Duration,
//     last_true_starts: Timestamp,
// }
//
// impl Patchable for DurationTrue {
//     type State = DurationTrueState;
//
//     fn patch_from(&mut self, state: Self::State) {
//         self.current_state = state.current_state;
//         self.accumulated_duration = state.accumulated_duration;
//         self.last_true_starts = state.last_true_starts;
//     }
// }

#[cfg(test)]
mod test {
    use lsp_runtime::signal_api::{Patchable, SignalMeasurement};

    use crate::test::{create_lsp_context_for_test, TestSignalBag};

    use super::DurationTrue;

    #[test]
    fn test_duration_prev_level() {
        let mut signal_bag = TestSignalBag::default();
        let mut duration_true = DurationTrue::default();
        let mut ctx = create_lsp_context_for_test();

        {
            let moment = ctx.next_event(&mut signal_bag).unwrap();
            assert_eq!(moment.timestamp(), 0);
            let mut uc = ctx.borrow_update_context();
            duration_true.update(&mut uc, &true);
            assert_eq!(duration_true.measure(&mut uc), 0);
        }

        {
            let moment = ctx.next_event(&mut signal_bag).unwrap();
            assert_eq!(moment.timestamp(), 1);
            let mut uc = ctx.borrow_update_context();
            duration_true.update(&mut uc, &true);
            assert_eq!(duration_true.measure(&mut uc), 1);
        }

        {
            let moment = ctx.next_event(&mut signal_bag).unwrap();
            assert_eq!(moment.timestamp(), 2);
            let mut uc = ctx.borrow_update_context();
            duration_true.update(&mut uc, &true);
            assert_eq!(duration_true.measure(&mut uc), 2);
        }

        {
            let moment = ctx.next_event(&mut signal_bag).unwrap();
            assert_eq!(moment.timestamp(), 3);
            let mut uc = ctx.borrow_update_context();
            duration_true.update(&mut uc, &false);
            assert_eq!(duration_true.measure(&mut uc), 3);
        }

        {
            let moment = ctx.next_event(&mut signal_bag).unwrap();
            assert_eq!(moment.timestamp(), 4);
            let mut uc = ctx.borrow_update_context();
            duration_true.update(&mut uc, &false);
            assert_eq!(duration_true.measure(&mut uc), 3);
        }

        {
            let moment = ctx.next_event(&mut signal_bag).unwrap();
            assert_eq!(moment.timestamp(), 5);
            let mut uc = ctx.borrow_update_context();
            duration_true.update(&mut uc, &true);
            assert_eq!(duration_true.measure(&mut uc), 3);
        }

        {
            let moment = ctx.next_event(&mut signal_bag).unwrap();
            assert_eq!(moment.timestamp(), 6);
            let mut uc = ctx.borrow_update_context();
            duration_true.update(&mut uc, &true);
            assert_eq!(duration_true.measure(&mut uc), 4);
        }

        let state = duration_true.to_state();
        let mut init_duration_true = DurationTrue::default();
        init_duration_true.patch(&state);
        assert_eq!(state, init_duration_true.to_state());
    }
}
