use std::marker::PhantomData;
use crate::{InternalEventQueue, Timestamp, Moment, multipeek::MultiPeek};

/// Some type that contains timestamp information.
/// Typically, we abstract an event taken from outside as this trait
/// and the context is responsible assemble the simutanious event into
/// the global input state
pub trait WithTimestamp {
    fn timestamp(&self) -> Timestamp;
}

/// The global input state which is applying the incoming event as patch
/// to the state and this is the external input type of the LSP system.
pub trait InputState: Clone + Default {
    type Event;
    /// Patch a event to the state
    fn patch(&mut self, patch: Self::Event);

    /// Determine if the input states need to trigger a measurement
    fn should_measure(&self) -> bool {
        false
    }
}

/// The global context of a LSP system. This type is responsible for the follwoing things:
/// 1. Take the ownership of a event queue which contains all the pending internal event
/// 2. Assemble events into valid global state
/// 3. Controlls the iteration of the LSP main loop
pub struct LspContext<I:Iterator, S> 
{
    frontier: Timestamp,
    iter: MultiPeek<I>,
    queue : InternalEventQueue,
    _phantom: PhantomData<S>,
}

pub struct UpdateContext<'a, I: Iterator> {
    queue: &'a mut InternalEventQueue,
    frontier: Timestamp,
    iter: &'a mut MultiPeek<I>,
}

impl <'a, I:Iterator> UpdateContext<'a, I> {
    pub fn schedule_measurement(&mut self, time_diff: Timestamp) {
        let scheduled_time = self.frontier.saturating_add(time_diff);
        self.queue.schedule_measurement(scheduled_time);
    }
    pub fn schedule_signal_update(&mut self, time_diff: Timestamp) {
        let scheduled_time = self.frontier.saturating_add(time_diff);
        self.queue.schedule_signal_update(scheduled_time);
    }
    pub fn peek_fold<U, F>(&mut self, init: U, func: F) -> U
    where
       F: FnMut(&U, &I::Item)  -> Option<U> 
    {
        self.iter.peek_fold(init, func)
    }
    pub fn frontier(&self) -> Timestamp {
        self.frontier
    }
}

impl <I, E, S> LspContext<I, S>
where
    E: WithTimestamp,
    S: InputState<Event = E>,
    I: Iterator<Item = E>,
{
    pub fn new(iter: I) -> Self {
        Self::with_queue(iter, InternalEventQueue::new())
    }

    pub fn with_queue(iter: I, queue: InternalEventQueue) -> Self {
        Self {
            iter: MultiPeek::from_iter(iter),
            queue,
            frontier: 0,
            _phantom: PhantomData,
        }
    }

    pub fn into_queue(self) -> InternalEventQueue {
        self.queue
    }

    pub fn borrow_update_context(&mut self) -> UpdateContext<I> {
        UpdateContext { 
            queue:  &mut self.queue, 
            frontier: self.frontier,
            iter: &mut self.iter,
        }
    }

    fn assemble_next_state(&mut self, timestamp: Timestamp, state: &mut S) {
        while let Some(ts) = self.iter.peek().map(|p| p.timestamp()) {
            if ts != timestamp {
                break;
            }
            let event = self.iter.next().unwrap();
            state.patch(event);
        }
    }

    pub fn next_event(&mut self, state_buf: &mut S) -> Option<Moment> {
        let external_frontier = self.iter.peek().map_or(Timestamp::MAX, |e| e.timestamp());
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