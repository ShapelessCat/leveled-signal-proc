use serde::Serialize;

use lsp_runtime::context::UpdateContext;
use lsp_runtime::signal_api::{Patchable, SignalMeasurement};
use lsp_runtime::{Duration, Timestamp};

/// Measure the duration from the start of the current level.
#[derive(Clone, Default, Debug, Serialize, Patchable)]
pub struct DurationOfCurrentLevel<T> {
    current_level_start: Timestamp,
    #[serde(skip_serializing_if = "Option::is_none")]
    current_level: Option<T>,
}

impl<'a, I, T> SignalMeasurement<'a, I> for DurationOfCurrentLevel<T>
where
    I: Iterator,
    T: Clone + PartialEq + Serialize,
{
    type Input = T;
    type Output = Duration;

    fn update(&mut self, ctx: &mut UpdateContext<I>, input: &'a Self::Input) {
        if self.current_level.iter().all(|ll| *input != *ll) {
            self.current_level = Some(input.clone());
            self.current_level_start = ctx.frontier();
        }
    }

    fn measure(&self, ctx: &mut UpdateContext<I>) -> Self::Output {
        if self.current_level.is_none() {
            0
        } else {
            ctx.frontier() - self.current_level_start
        }
    }
}

// #[derive(Deserialize)]
// pub struct DurationOfCurrentLevelState<T> {
//     current_level_start: Timestamp,
//     current_level: Option<T>,
// }
//
// impl<T> Patchable for DurationOfCurrentLevel<T>
// where
//     T: Serialize + DeserializeOwned,
// {
//     type State = DurationOfCurrentLevelState<T>;
//
//     fn patch_from(&mut self, state: Self::State) {
//         self.current_level = state.current_level;
//         self.current_level_start = state.current_level_start;
//     }
// }

#[cfg(test)]
mod test {
    use lsp_runtime::signal_api::{Patchable, SignalMeasurement};

    use crate::test::{create_lsp_context_for_test, TestSignalBag};

    use super::DurationOfCurrentLevel;

    #[test]
    fn test_duration_since_last_level() {
        let mut signal_bag = TestSignalBag::default();
        let mut duration_of_current_level = DurationOfCurrentLevel::default();
        let mut ctx = create_lsp_context_for_test();

        {
            let moment = ctx.next_event(&mut signal_bag).unwrap();
            assert_eq!(moment.timestamp(), 0);
            let mut uc = ctx.borrow_update_context();
            duration_of_current_level.update(&mut uc, &true);
            assert_eq!(duration_of_current_level.measure(&mut uc), 0);
        }

        {
            let moment = ctx.next_event(&mut signal_bag).unwrap();
            assert_eq!(moment.timestamp(), 1);
            let mut uc = ctx.borrow_update_context();
            duration_of_current_level.update(&mut uc, &true);
            assert_eq!(duration_of_current_level.measure(&mut uc), 1);
        }

        {
            let moment = ctx.next_event(&mut signal_bag).unwrap();
            assert_eq!(moment.timestamp(), 2);
            let mut uc = ctx.borrow_update_context();
            duration_of_current_level.update(&mut uc, &false);
            assert_eq!(duration_of_current_level.measure(&mut uc), 0);
        }

        {
            let moment = ctx.next_event(&mut signal_bag).unwrap();
            assert_eq!(moment.timestamp(), 3);
            let mut uc = ctx.borrow_update_context();
            duration_of_current_level.update(&mut uc, &false);
            assert_eq!(duration_of_current_level.measure(&mut uc), 1);
        }

        {
            let moment = ctx.next_event(&mut signal_bag).unwrap();
            assert_eq!(moment.timestamp(), 4);
            let mut uc = ctx.borrow_update_context();
            duration_of_current_level.update(&mut uc, &true);
            assert_eq!(duration_of_current_level.measure(&mut uc), 0);
        }

        {
            let moment = ctx.next_event(&mut signal_bag).unwrap();
            assert_eq!(moment.timestamp(), 5);
            let mut uc = ctx.borrow_update_context();
            duration_of_current_level.update(&mut uc, &true);
            assert_eq!(duration_of_current_level.measure(&mut uc), 1);
        }

        let state = duration_of_current_level.to_state();
        let mut init_duration_of_current_level = DurationOfCurrentLevel::<bool>::default();
        init_duration_of_current_level.patch(&state);
        assert_eq!(state, init_duration_of_current_level.to_state());
    }
}
