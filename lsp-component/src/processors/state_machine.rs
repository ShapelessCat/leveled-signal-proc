use std::{collections::VecDeque, fmt::Debug, marker::PhantomData};

use serde::{Deserialize, Serialize};

use lsp_runtime::{Duration, signal::SignalProcessor, Timestamp, UpdateContext};

/// A state machine is a signal processor that maintains a state machine internally.
/// The state transition is defined as a lambda function passed in when construction.
/// The state transition is triggered when the control input gets changed.
/// The output is simply the current internal state.
#[derive(Deserialize, Serialize)]
pub struct StateMachine<Input, State: Clone, TransitionFunc, Trigger> {
    state: State,
    transition: TransitionFunc,
    last_trigger_value: Trigger,
    _phantom: PhantomData<Input>,
}

impl<I, S: Debug + Clone, F, T: Debug> Debug for StateMachine<I, S, F, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StateMachine")
            .field("state", &self.state)
            .field("last_trigger_value", &self.last_trigger_value)
            .field("_phantom", &self._phantom)
            .finish()
    }
}

impl<I, S: Clone, F, T: Default> StateMachine<I, S, F, T> {
    pub fn new(initial_state: S, transition: F) -> Self
    where
        F: Fn(&S, &I) -> S,
    {
        Self {
            state: initial_state,
            transition,
            last_trigger_value: Default::default(),
            _phantom: PhantomData,
        }
    }
}

impl<'a, EventIterator, Input, State, Transition, Trigger> SignalProcessor<'a, EventIterator>
    for StateMachine<Input, State, Transition, Trigger>
where
    Transition: Fn(&State, &Input) -> State,
    EventIterator: Iterator,
    State: Clone,
    Trigger: Eq + Clone + 'a,
    Input: 'a,
{
    type Input = (&'a Trigger, &'a Input);

    type Output = State;

    fn update(
        &mut self,
        _: &mut UpdateContext<EventIterator>,
        (trigger, input): Self::Input,
    ) -> Self::Output {
        if trigger != &self.last_trigger_value {
            self.state = (self.transition)(&self.state, input);
            self.last_trigger_value = trigger.clone();
        }
        self.state.clone()
    }
}

pub struct SlidingWindow<Input, EmitFunc, Trigger, Output> {
    queue: VecDeque<Input>,
    emit_func: EmitFunc,
    last_trigger_value: Trigger,
    last_dequeued_value: Input,
    _phantom: PhantomData<Output>,
}

impl<I: Default, F, T: Default, O> SlidingWindow<I, F, T, O> {
    pub fn new(emit_func: F, window_size: usize, init_value: I) -> Self
    where
        F: Fn(&VecDeque<I>, &I) -> O,
    {
        Self {
            queue: VecDeque::with_capacity(window_size),
            emit_func,
            last_trigger_value: Default::default(),
            last_dequeued_value: init_value,
            _phantom: PhantomData,
        }
    }
}

impl<'a, Input, EmitFunc, Iter, Trigger, Output> SignalProcessor<'a, Iter>
    for SlidingWindow<Input, EmitFunc, Trigger, Output>
where
    EmitFunc: Fn(&VecDeque<Input>, &Input) -> Output,
    Iter: Iterator,
    Output: Clone,
    Trigger: Eq + Clone + 'a,
    Input: 'a + Clone,
{
    type Input = (&'a Trigger, &'a Input);

    type Output = Output;

    fn update(
        &mut self,
        _: &mut UpdateContext<Iter>,
        (trigger, input): Self::Input,
    ) -> Self::Output {
        if trigger != &self.last_trigger_value {
            if self.queue.len() == self.queue.capacity() {
                self.last_dequeued_value = self.queue.pop_front().unwrap();
            }
            self.queue.push_back(input.clone());
            self.last_trigger_value = trigger.clone();
        }
        (self.emit_func)(&self.queue, &self.last_dequeued_value)
    }
}

pub struct SlidingTimeWindow<Input, EmitFunc, Trigger, Output> {
    queue: VecDeque<(Input, Timestamp)>,
    time_window_size: Duration,
    emit_func: EmitFunc,
    last_trigger_value: Trigger,
    last_dequeued_value: Input,
    _phantom: PhantomData<Output>,
}

impl<I: Default, F, T: Default, O> SlidingTimeWindow<I, F, T, O> {
    pub fn new(emit_func: F, time_window_size: Duration, init_value: I) -> Self
    where
        F: Fn(&VecDeque<(I, Timestamp)>, &I) -> O,
    {
        Self {
            queue: VecDeque::new(),
            time_window_size,
            emit_func,
            last_trigger_value: Default::default(),
            last_dequeued_value: init_value,
            _phantom: PhantomData,
        }
    }
}

impl<'a, Input, EmitFunc, Iter, Trigger, Output> SignalProcessor<'a, Iter>
    for SlidingTimeWindow<Input, EmitFunc, Trigger, Output>
where
    EmitFunc: Fn(&VecDeque<(Input, Timestamp)>, &Input) -> Output,
    Iter: Iterator,
    Output: Clone,
    Trigger: Eq + Clone + 'a,
    Input: 'a + Clone,
{
    type Input = (&'a Trigger, &'a Input);

    type Output = Output;

    fn update(
        &mut self,
        ctx: &mut UpdateContext<Iter>,
        (trigger, input): Self::Input,
    ) -> Self::Output {
        while let Some((_, timestamp)) = self.queue.front() {
            if ctx.frontier() - timestamp >= self.time_window_size {
                self.last_dequeued_value = self.queue.pop_front().unwrap().0;
            } else {
                break;
            }
        }
        ctx.schedule_signal_update(self.time_window_size);
        if trigger != &self.last_trigger_value {
            self.queue.push_back((input.clone(), ctx.frontier()));
            self.last_trigger_value = trigger.clone();
        }
        (self.emit_func)(&self.queue, &self.last_dequeued_value)
    }
}
