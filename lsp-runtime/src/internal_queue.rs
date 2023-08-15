use crate::{Moment, Timestamp};
use std::{collections::BinaryHeap, cmp::Reverse};

/// The queue sorting internal events
///
/// A event is a moment that leveled signals may be changed.
/// In most case, leveled signal is changed due to the external input or
/// in our terms, external event.
/// But in some cases, for instance, a sliding window, the leveled signal
/// spontanously change its value. To handle this case, we introduced the
/// concept of internal event, which isn't triggered by any external event,
/// but scheduled whenever the signal processor needs a recompute.
///
/// Also, we handle the measurement request as a internal event.
pub struct InternalEventQueue {
    queue: BinaryHeap<Reverse<Moment>>,
}

impl InternalEventQueue {
    pub fn new() -> Self {
        InternalEventQueue {
            queue: BinaryHeap::new(),
        }
    }

    pub fn schedule_signal_update(&mut self, timestamp: Timestamp) {
        self.queue.push(Reverse(Moment::signal_update(timestamp)));
    }

    pub fn schedule_measurement(&mut self, timestamp: Timestamp) {
        self.queue.push(Reverse(Moment::measurement(timestamp)))
    }

    pub fn earliest_scheduled_time(&self) -> Timestamp {
        self.queue.peek().map_or(Timestamp::MAX, |Reverse(e)| e.timestamp())
    }

    pub fn pop(&mut self) -> Option<Moment> {
        if let Some(Reverse(mut ret)) = self.queue.pop() {
            while let Some(Reverse(event)) = self.queue.peek() {
                if let Some(merged) = ret.merge(event) {
                    ret = merged;
                } else {
                    break;
                }
                self.queue.pop();
            }
            Some(ret)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test{
    use super::*;
    #[test]
    fn test_internal_event_queue() {
        let mut queue = InternalEventQueue::new();
        queue.schedule_signal_update(2);
        queue.schedule_measurement(2);
        queue.schedule_signal_update(1);
        queue.schedule_measurement(10);
        assert!(queue.earliest_scheduled_time() == 1);
        assert!(queue.pop().unwrap() == Moment::signal_update(1));
        assert!(queue.earliest_scheduled_time() == 2);
        let moment = queue.pop().unwrap();
        assert!(moment.timestamp() == 2);
        assert!(moment.should_take_measurements());
        assert!(moment.should_update_signals());
        queue.schedule_measurement(5);
        assert!(queue.earliest_scheduled_time() == 5);
        assert!(queue.pop().unwrap() == Moment::measurement(5));
        assert!(queue.pop().unwrap() == Moment::measurement(10));
        assert!(queue.pop().is_none());
    }
}