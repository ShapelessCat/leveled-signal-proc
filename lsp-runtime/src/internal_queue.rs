use crate::{Moment, Timestamp};
use std::collections::BinaryHeap;

/// The queue that sorts internal events
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
    queue: BinaryHeap<Moment>,
}

impl InternalEventQueue {
    pub fn new() -> Self {
        InternalEventQueue {
            queue: BinaryHeap::new(),
        }
    }

    pub fn schedule_signal_update(&mut self, timestamp: Timestamp) {
        self.queue.push(Moment::signal_update(timestamp));
    }

    pub fn schedule_measurement(&mut self, timestamp: Timestamp) {
        self.queue.push(Moment::measurement(timestamp))
    }

    pub fn earliest_scheduled_time(&self) -> Timestamp {
        self.queue.peek().map_or(Timestamp::MAX, |e| e.timestamp())
    }

    pub fn pop(&mut self) -> Option<Moment> {
        if let Some(mut ret) = self.queue.pop() {
            while let Some(event) = self.queue.peek() {
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
