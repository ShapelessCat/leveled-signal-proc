use std::{collections::BinaryHeap, ops::AddAssign};
use crate::Timestamp;

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub enum UpdateScope {
    Measure,
    Full,
}

impl AddAssign<UpdateScope> for UpdateScope {
    fn add_assign(&mut self, rhs: UpdateScope) {
        if *self == rhs {
            return;
        }
        *self = UpdateScope::Full;
    }
}

/// When the signal processing is running, there may be some update that isn't
/// triggered by the input event. For example, if we have a node which is maintaining
/// a sliding window, for each given event, we need to schedule an update when the event 
/// is moving out of the window.
/// 
/// This scheduled update request is for this purpose. When the input is feed into a
/// component the component may schedule an update event which will wake the signal processor
/// when the timestamp is advanced beyond the scheduled time.
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub struct UpdateRequest{
    pub scheduled_time: Timestamp,
    pub scope: UpdateScope,
}

/// The context of the 
pub struct UpdateQueue {
    now: Timestamp,
    update_request: BinaryHeap<UpdateRequest>,
}

impl UpdateQueue {
    pub fn new() -> Self {
        UpdateQueue { 
            now: 0, 
            update_request: Default::default() 
        }
    }
    /// Schedule an update wakeup after time_diff period of time
    fn schedule_update(&mut self, time_diff: Timestamp, scope: UpdateScope) {
        self.update_request.push(UpdateRequest { 
            scheduled_time: self.now + time_diff, 
            scope, 
        })
    }

    pub fn schedule_signal_update(&mut self, time_diff: Timestamp) {
        self.schedule_update(time_diff, UpdateScope::Full);
    }

    pub fn schedule_measure_update(&mut self, time_diff: Timestamp) {
        self.schedule_update(time_diff, UpdateScope::Measure);
    }

    pub fn set_epoch(&mut self, timestamp: Timestamp) {
        self.now = timestamp;
    }

    pub fn pop(&mut self, ts_cutoff: Timestamp) -> Option<UpdateRequest> {
        let next_event = self.update_request.peek()?;
        if next_event.scheduled_time > ts_cutoff {
            return None;
        }
        let mut next_event = self.update_request.pop().unwrap();
        while let Some(peeked) = self.update_request.peek() {
            if peeked.scheduled_time != next_event.scheduled_time {
                break;
            }
            next_event.scope += peeked.scope;
            self.update_request.pop();
        }
        Some(next_event)
    }

}