use serde::Serialize;

use lsp_runtime::context::UpdateContext;
use lsp_runtime::signal_api::SignalMeasurement;
use lsp_runtime::{Duration, Timestamp};

#[derive(Clone, Default, Debug, Serialize)]
pub struct DurationOfCurrentLevel<T> {
    current_level_start: Timestamp,
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

#[cfg(test)]
mod test {
    use lsp_runtime::signal_api::SignalMeasurement;

    use crate::test::{create_lsp_context_for_test, TestSignalBag};

    use super::DurationOfCurrentLevel;

    #[test]
    fn test_duration_since_last_level() {
        let mut signal_bag = TestSignalBag::default();
        let mut duration_since_last_level = DurationOfCurrentLevel::default();
        let mut ctx = create_lsp_context_for_test();

        {
            let moment = ctx.next_event(&mut signal_bag).unwrap();
            assert_eq!(moment.timestamp(), 0);
            let mut uc = ctx.borrow_update_context();
            duration_since_last_level.update(&mut uc, &true);
            assert_eq!(duration_since_last_level.measure(&mut uc), 0);
        }

        {
            let moment = ctx.next_event(&mut signal_bag).unwrap();
            assert_eq!(moment.timestamp(), 1);
            let mut uc = ctx.borrow_update_context();
            duration_since_last_level.update(&mut uc, &true);
            assert_eq!(duration_since_last_level.measure(&mut uc), 1);
        }

        {
            let moment = ctx.next_event(&mut signal_bag).unwrap();
            assert_eq!(moment.timestamp(), 2);
            let mut uc = ctx.borrow_update_context();
            duration_since_last_level.update(&mut uc, &false);
            assert_eq!(duration_since_last_level.measure(&mut uc), 0);
        }

        {
            let moment = ctx.next_event(&mut signal_bag).unwrap();
            assert_eq!(moment.timestamp(), 3);
            let mut uc = ctx.borrow_update_context();
            duration_since_last_level.update(&mut uc, &false);
            assert_eq!(duration_since_last_level.measure(&mut uc), 1);
        }

        {
            let moment = ctx.next_event(&mut signal_bag).unwrap();
            assert_eq!(moment.timestamp(), 4);
            let mut uc = ctx.borrow_update_context();
            duration_since_last_level.update(&mut uc, &true);
            assert_eq!(duration_since_last_level.measure(&mut uc), 0);
        }

        {
            let moment = ctx.next_event(&mut signal_bag).unwrap();
            assert_eq!(moment.timestamp(), 5);
            let mut uc = ctx.borrow_update_context();
            duration_since_last_level.update(&mut uc, &true);
            assert_eq!(duration_since_last_level.measure(&mut uc), 1);
        }
    }
}
