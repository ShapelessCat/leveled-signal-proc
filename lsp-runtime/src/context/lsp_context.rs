use std::marker::PhantomData;

use serde::{Deserialize, Serialize};

use crate::signal_api::Patchable;
use crate::{Duration, Moment, Timestamp};

use super::multipeek::MultiPeekState;
use super::{InputSignalBag, InternalEventQueue, MultiPeek, WithTimestamp};

/// The global context of an LSP system. This type is responsible for the following things:
/// 1. Take the ownership of an event queue which contains all the pending internal events
/// 2. Assemble events into valid global state
/// 3. Control the iteration of the LSP main iteration
#[derive(Serialize)]
#[serde(bound = "")]
pub struct LspContext<InputIter: Iterator, InputSignalBagType> {
    frontier: Timestamp,
    #[serde(rename = "iter_state")]
    iter: MultiPeek<InputIter>,
    queue: InternalEventQueue,
    merge_simultaneous_moments: bool,
    #[serde(skip)]
    _phantom_data: PhantomData<InputSignalBagType>,
}

#[derive(Default, Deserialize)]
pub struct LspContextState {
    frontier: Timestamp,
    iter_state: MultiPeekState,
    queue: InternalEventQueue,
    merge_simultaneous_moments: bool,
}

impl<InputIter: Iterator, InputSignalBagType> Patchable
    for LspContext<InputIter, InputSignalBagType>
{
    type State = LspContextState;

    fn patch_from(&mut self, state: Self::State) {
        self.frontier = state.frontier;
        self.iter.patch_from(state.iter_state);
        self.queue = state.queue;
        self.merge_simultaneous_moments = state.merge_simultaneous_moments;
    }
}

#[derive(Serialize)]
#[serde(bound = "")]
pub struct UpdateContext<'a, InputIter: Iterator> {
    queue: &'a mut InternalEventQueue,
    #[serde(rename = "iter_state")]
    iter: &'a mut MultiPeek<InputIter>,
    frontier: Timestamp,
    merge_simultaneous_moments: bool,
}

impl<InputIter: Iterator> UpdateContext<'_, InputIter> {
    pub fn offset(&self) -> usize {
        self.iter.offset()
    }

    pub fn set_current_update_group(&mut self, _group_id: u32) {
        // Dummy implementation reserved for partial update
    }

    pub fn schedule_measurement(&mut self, time_diff: Duration) {
        let scheduled_time = self.frontier.saturating_add(time_diff);
        self.queue.schedule_measurement(scheduled_time);
    }

    pub fn schedule_signal_update(&mut self, time_diff: Duration) {
        let scheduled_time = self.frontier.saturating_add(time_diff);
        self.queue.schedule_signal_update(scheduled_time);
    }

    pub fn peek_fold<U, F>(&mut self, init: U, func: F) -> U
    where
        F: FnMut(&U, &InputIter::Item) -> Option<U>,
    {
        self.iter.peek_fold(init, func)
    }

    pub fn frontier(&self) -> Timestamp {
        self.frontier
    }
}

impl<InputIter, InputType, SignalBag> LspContext<InputIter, SignalBag>
where
    InputType: WithTimestamp,
    SignalBag: InputSignalBag<Input = InputType>,
    InputIter: Iterator<Item = InputType>,
{
    pub fn new(iter: InputIter, merge_simultaneous_moments: bool) -> Self {
        Self::with_queue(
            iter,
            InternalEventQueue::default(),
            merge_simultaneous_moments,
        )
    }

    pub fn with_queue(
        iter: InputIter,
        queue: InternalEventQueue,
        merge_simultaneous_moments: bool,
    ) -> Self {
        Self {
            iter: MultiPeek::from(iter),
            queue,
            frontier: 0,
            merge_simultaneous_moments,
            _phantom_data: PhantomData,
        }
    }

    pub fn into_queue(self) -> InternalEventQueue {
        self.queue
    }

    #[inline(always)]
    pub fn borrow_update_context(&mut self) -> UpdateContext<InputIter> {
        UpdateContext {
            queue: &mut self.queue,
            frontier: self.frontier,
            iter: &mut self.iter,
            merge_simultaneous_moments: self.merge_simultaneous_moments,
        }
    }

    #[inline(always)]
    fn assemble_next_state(&mut self, timestamp: Timestamp, state: &mut SignalBag) {
        if self.merge_simultaneous_moments {
            while let Some(ts) = self.iter.peek().map(WithTimestamp::timestamp) {
                if ts != timestamp {
                    break;
                }
                let event = self.iter.next().unwrap();
                state.patch(event);
            }
        } else if let Some(ts) = self.iter.peek().map(WithTimestamp::timestamp) {
            if ts == timestamp {
                let event = self.iter.next().unwrap();
                state.patch(event);
            }
        }
    }

    #[inline(always)]
    pub fn next_event(&mut self, state_buf: &mut SignalBag) -> Option<Moment> {
        // If there's no more output, we just exit the scanning loop anyway
        let external_frontier = self.iter.peek().map(WithTimestamp::timestamp)?;
        let internal_frontier = self.queue.earliest_scheduled_time();

        if external_frontier != Timestamp::MAX && external_frontier <= internal_frontier {
            self.frontier = external_frontier;
            self.assemble_next_state(external_frontier, state_buf);
            let mut ret = Moment::signal_update(external_frontier);
            if state_buf.should_measure() {
                ret = ret.merge(&Moment::measurement(ret.timestamp())).unwrap();
            }
            if external_frontier == internal_frontier {
                if let Some(internal_event) = self.queue.pop() {
                    ret = ret.merge(&internal_event).unwrap();
                }
            }
            Some(ret)
        } else {
            self.frontier = internal_frontier;
            self.queue.pop()
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    struct TestInput {
        timestamp: Timestamp,
        value: u32,
    }

    impl WithTimestamp for TestInput {
        fn timestamp(&self) -> Timestamp {
            self.timestamp
        }
    }

    #[derive(Clone, Debug, PartialEq, Default)]
    struct TestSignalBag {
        value: u32,
    }

    impl InputSignalBag for TestSignalBag {
        type Input = TestInput;

        fn patch(&mut self, patch: Self::Input) {
            self.value = patch.value;
        }
    }

    fn create_test_context(
        merge_simultaneous_moments: bool,
    ) -> LspContext<<Vec<TestInput> as IntoIterator>::IntoIter, TestSignalBag> {
        LspContext::new(
            vec![
                TestInput {
                    timestamp: 0,
                    value: 1,
                },
                TestInput {
                    timestamp: 0,
                    value: 2,
                },
                TestInput {
                    timestamp: 1,
                    value: 3,
                },
                TestInput {
                    timestamp: 20,
                    value: 4,
                },
            ]
            .into_iter(),
            merge_simultaneous_moments,
        )
    }

    #[test]
    fn test_external_event_assemble() {
        let mut context = create_test_context(true);

        let mut state = TestSignalBag { value: 0 };

        assert_eq!(
            context.next_event(&mut state),
            Some(Moment::signal_update(0))
        );
        assert_eq!(state.value, 2);
        assert_eq!(
            context.next_event(&mut state),
            Some(Moment::signal_update(1))
        );
        assert_eq!(state.value, 3);
        assert_eq!(
            context.next_event(&mut state),
            Some(Moment::signal_update(20))
        );
        assert_eq!(state.value, 4);
        assert_eq!(context.next_event(&mut state), None);
    }

    #[test]
    fn test_internal_event_queue() {
        let mut context = create_test_context(true);

        let mut state = TestSignalBag { value: 0 };

        let mut uc = context.borrow_update_context();
        uc.schedule_measurement(10);

        assert_eq!(
            context.next_event(&mut state),
            Some(Moment::signal_update(0))
        );
        assert_eq!(state.value, 2);
        assert_eq!(
            context.next_event(&mut state),
            Some(Moment::signal_update(1))
        );
        assert_eq!(state.value, 3);
        assert_eq!(
            context.next_event(&mut state),
            Some(Moment::measurement(10))
        );
        assert_eq!(
            context.next_event(&mut state),
            Some(Moment::signal_update(20))
        );
        assert_eq!(state.value, 4);
        assert_eq!(context.next_event(&mut state), None);
    }
}
