use std::{marker::PhantomData, iter::Peekable};

use crate::{UpdateQueue, Timestamp, update_queue::UpdateScope};


pub trait WithTimestamp {
    fn timestamp(&self) -> Timestamp;
}

pub struct Context<I, E : WithTimestamp> 
where
    I: Iterator<Item = E>,
    E: WithTimestamp,
{
    iter: Peekable<I>,
    update_queue : UpdateQueue,
    _phantom: PhantomData<E>,
}

pub struct Event<E> {
    timestamp: Timestamp,
    input: Option<E>,
    update_request: UpdateScope,
}

impl <E> Event<E> {
    pub fn timestamp(&self) -> Timestamp {
        self.timestamp
    }
    pub fn input(&self) -> Option<&E> {
        self.input.as_ref()
    }
    pub fn should_update(&self) -> bool {
        matches!(self.update_request, UpdateScope::Full)
    }
    pub fn should_measure(&self) -> bool {
        true
    }
    pub fn into_input(self) -> Option<E> {
        self.input
    }
}

impl <I, E> Context<I, E>
where
    I: Iterator<Item = E>,
    E: WithTimestamp,
{
    pub fn new(iter: I) -> Self {
        Self::with_update_queue(iter, UpdateQueue::new())
    }

    pub fn with_update_queue(iter: I, update_queue: UpdateQueue) -> Self {
        Self {
            iter: iter.peekable(),
            update_queue,
            _phantom: PhantomData
        }
    }

    pub fn into_update_queue(self) -> UpdateQueue {
        self.update_queue

    }

    pub fn update_queue(&self) -> &UpdateQueue {
        &self.update_queue
    }

    pub fn update_queue_mut(&mut self) -> &mut UpdateQueue {
        &mut self.update_queue
    }

    pub fn next_event(&mut self) -> Option<Event<E>> {
        let next_key_instant = self.iter.peek().map_or(Timestamp::MAX, |e| e.timestamp());
        let ret = if let Some(update_request) = self.update_queue.pop(next_key_instant) {
            Some(Event {
                timestamp: update_request.scheduled_time,
                input: if next_key_instant == update_request.scheduled_time { self.iter.next() } else { None },
                update_request: update_request.scope,
            })
        } else {
            self.iter.next().map(|raw_input| Event { 
                timestamp: raw_input.timestamp(), 
                input: Some(raw_input), 
                update_request: UpdateScope::Full 
            })
        };
        ret
    }
}