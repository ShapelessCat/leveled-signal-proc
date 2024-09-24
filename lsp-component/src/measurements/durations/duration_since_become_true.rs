use serde::Serialize;

use lsp_runtime::context::UpdateContext;
use lsp_runtime::signal_api::{Patchable, SignalMeasurement};
use lsp_runtime::{Duration, Timestamp};

/// Measure the duration since the signal become `true`.
/// It is easy get some conclusion based on this description:
/// - If the current level is `true`, the measurement result is greater than or equal to 0.
/// - If the current level is `false`, the measurement result 0.
#[derive(Clone, Default, Serialize, Patchable)]
pub struct DurationSinceBecomeTrue {
    last_input: bool,
    last_assignment_timestamp: Timestamp,
}

impl<'a, I: Iterator> SignalMeasurement<'a, I> for DurationSinceBecomeTrue {
    type Input = bool;
    type Output = Duration;

    fn update(&mut self, ctx: &mut UpdateContext<I>, input: &'a Self::Input) {
        if *input != self.last_input {
            self.last_input = *input;
            self.last_assignment_timestamp = ctx.frontier();
        }
    }

    fn measure(&self, ctx: &mut UpdateContext<I>) -> Self::Output {
        if self.last_input {
            ctx.frontier() - self.last_assignment_timestamp
        } else {
            0
        }
    }
}

// #[derive(Deserialize)]
// pub struct DurationSinceBecomeTrueState {
//     last_input: bool,
//     last_assignment_timestamp: Timestamp,
// }
//
// impl Patchable for DurationSinceBecomeTrue {
//     type State = DurationSinceBecomeTrueState;
//
//     fn patch_from(&mut self, state: Self::State) {
//         self.last_input = state.last_input;
//         self.last_assignment_timestamp = state.last_assignment_timestamp;
//     }
// }

#[cfg(test)]
mod test {
    use lsp_runtime::signal_api::{Patchable, SignalMeasurement};

    use crate::test::{create_lsp_context_for_test, TestSignalBag};

    use super::DurationSinceBecomeTrue;

    #[test]
    fn test_duration_since_become_true() {
        let mut signal_bag = TestSignalBag::default();
        let mut duration_since_become_true = DurationSinceBecomeTrue::default();
        let mut ctx = create_lsp_context_for_test();

        {
            let moment = ctx.next_event(&mut signal_bag).unwrap();
            assert_eq!(moment.timestamp(), 0);
            let mut uc = ctx.borrow_update_context();
            duration_since_become_true.update(&mut uc, &true);
            assert_eq!(duration_since_become_true.measure(&mut uc), 0);
        }

        {
            let moment = ctx.next_event(&mut signal_bag).unwrap();
            assert_eq!(moment.timestamp(), 1);
            let mut uc = ctx.borrow_update_context();
            duration_since_become_true.update(&mut uc, &true);
            assert_eq!(duration_since_become_true.measure(&mut uc), 1);
        }

        {
            let moment = ctx.next_event(&mut signal_bag).unwrap();
            assert_eq!(moment.timestamp(), 2);
            let mut uc = ctx.borrow_update_context();
            duration_since_become_true.update(&mut uc, &false);
            assert_eq!(duration_since_become_true.measure(&mut uc), 0);
        }

        {
            let moment = ctx.next_event(&mut signal_bag).unwrap();
            assert_eq!(moment.timestamp(), 3);
            let mut uc = ctx.borrow_update_context();
            duration_since_become_true.update(&mut uc, &false);
            assert_eq!(duration_since_become_true.measure(&mut uc), 0);
        }

        {
            let moment = ctx.next_event(&mut signal_bag).unwrap();
            assert_eq!(moment.timestamp(), 4);
            let mut uc = ctx.borrow_update_context();
            duration_since_become_true.update(&mut uc, &true);
            assert_eq!(duration_since_become_true.measure(&mut uc), 0);
        }

        {
            let moment = ctx.next_event(&mut signal_bag).unwrap();
            assert_eq!(moment.timestamp(), 5);
            let mut uc = ctx.borrow_update_context();
            duration_since_become_true.update(&mut uc, &true);
            assert_eq!(duration_since_become_true.measure(&mut uc), 1);
        }

        let state = duration_since_become_true.to_state();
        let mut init_duration_since_become_true = DurationSinceBecomeTrue::default();
        init_duration_since_become_true.patch(&state);
        assert_eq!(state, init_duration_since_become_true.to_state());
    }
}
