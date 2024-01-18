use serde::{Deserialize, Serialize};

use lsp_runtime::{signal::SignalProcessor, Duration, Timestamp, UpdateContext};

/// Note:
/// Although the duration of current level cannot be a measurement, as it's a function of time,
/// duration of previous level is a well defined signal -- duration of previous level is a known
/// value.
#[derive(Default, Debug, Deserialize, Serialize)]
pub struct DurationOfPreviousLevel<Level> {
    current_value: Level,
    current_value_since: Timestamp,
    output_buf: Timestamp,
}

impl<'a, I: Iterator, L: PartialEq + Clone> SignalProcessor<'a, I> for DurationOfPreviousLevel<L> {
    type Input = L;

    type Output = Duration;

    #[inline(always)]
    fn update(&mut self, ctx: &mut UpdateContext<I>, input: &'a Self::Input) -> Self::Output {
        if &self.current_value != input {
            self.output_buf = ctx.frontier() - self.current_value_since;
            self.current_value = input.clone();
            self.current_value_since = ctx.frontier();
        }
        self.output_buf
    }
}

#[cfg(test)]
mod test {
    use lsp_runtime::signal::SignalProcessor;

    use crate::test::{create_lsp_context_for_test, TestSignalBag};

    use super::DurationOfPreviousLevel;

    #[test]
    fn test_duration_prev_level() {
        let mut signal_bag = TestSignalBag::default();
        let mut node = DurationOfPreviousLevel::default();
        let mut ctx = create_lsp_context_for_test();

        let moment = ctx.next_event(&mut signal_bag).unwrap();
        assert_eq!(moment.timestamp(), 0);
        let mut uc = ctx.borrow_update_context();
        assert_eq!(node.update(&mut uc, &0), 0);
        drop(uc);

        let moment = ctx.next_event(&mut signal_bag).unwrap();
        assert_eq!(moment.timestamp(), 1);
        let mut uc = ctx.borrow_update_context();
        assert_eq!(node.update(&mut uc, &0), 0);
        drop(uc);

        let moment = ctx.next_event(&mut signal_bag).unwrap();
        assert_eq!(moment.timestamp(), 2);
        let mut uc = ctx.borrow_update_context();
        assert_eq!(node.update(&mut uc, &1), 2);
        drop(uc);

        let moment = ctx.next_event(&mut signal_bag).unwrap();
        assert_eq!(moment.timestamp(), 3);
        let mut uc = ctx.borrow_update_context();
        assert_eq!(node.update(&mut uc, &1), 2);
        drop(uc);

        let moment = ctx.next_event(&mut signal_bag).unwrap();
        assert_eq!(moment.timestamp(), 4);
        let mut uc = ctx.borrow_update_context();
        assert_eq!(node.update(&mut uc, &1), 2);
        drop(uc);

        let moment = ctx.next_event(&mut signal_bag).unwrap();
        assert_eq!(moment.timestamp(), 5);
        let mut uc = ctx.borrow_update_context();
        assert_eq!(node.update(&mut uc, &0), 3);
        drop(uc);
    }
}
