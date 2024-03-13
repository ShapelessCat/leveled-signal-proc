use std::cmp::Reverse;
use std::collections::BinaryHeap;

use crate::{Moment, Timestamp};

/// The queue sorting internal events.
///
/// An event is a moment that leveled signals may be changed.
/// In most case, leveled signal is changed due to the external input or in our terms, external
/// event, but in some cases, for instance, a sliding window, the leveled signal spontaneously
/// change its value. To handle this case, we introduced the concept of internal event, which isn't
/// triggered by any external event, but scheduled whenever the signal processor needs a recompute.
///
/// Also, we handle the measurement request as an internal event.
#[derive(Default)]
pub struct InternalEventQueue {
    queue: BinaryHeap<Reverse<Moment>>,
}

impl InternalEventQueue {
    pub fn schedule_signal_update(&mut self, timestamp: Timestamp) {
        self.queue.push(Reverse(Moment::signal_update(timestamp)));
    }

    pub fn schedule_measurement(&mut self, timestamp: Timestamp) {
        self.queue.push(Reverse(Moment::measurement(timestamp)))
    }

    pub fn earliest_scheduled_time(&self) -> Timestamp {
        self.queue
            .peek()
            .map_or(Timestamp::MAX, |Reverse(e)| e.timestamp())
    }

    pub fn pop(&mut self) -> Option<Moment> {
        let Reverse(mut ret) = self.queue.pop()?;
        while let Some(Reverse(event)) = self.queue.peek() {
            if let Some(merged) = ret.merge(event) {
                ret = merged;
            } else {
                break;
            }
            self.queue.pop();
        }
        Some(ret)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_internal_event_queue() {
        let mut queue = InternalEventQueue::default();
        queue.schedule_signal_update(2);
        queue.schedule_measurement(2);
        queue.schedule_signal_update(1);
        queue.schedule_measurement(10);
        assert_eq!(queue.earliest_scheduled_time(), 1);
        assert_eq!(queue.pop().unwrap(), Moment::signal_update(1));
        assert_eq!(queue.earliest_scheduled_time(), 2);
        let moment = queue.pop().unwrap();
        assert_eq!(moment.timestamp(), 2);
        assert!(moment.should_take_measurements());
        assert!(moment.should_update_signals());
        queue.schedule_measurement(5);
        assert_eq!(queue.earliest_scheduled_time(), 5);
        assert_eq!(queue.pop().unwrap(), Moment::measurement(5));
        assert_eq!(queue.pop().unwrap(), Moment::measurement(10));
        assert!(queue.pop().is_none());
    }
}
