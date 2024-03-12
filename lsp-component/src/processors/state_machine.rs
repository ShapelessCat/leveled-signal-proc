use std::{collections::VecDeque, fmt::Debug, marker::PhantomData};

use serde::Serialize;

use lsp_runtime::context::UpdateContext;
use lsp_runtime::signal_api::SignalProcessor;
use lsp_runtime::{Duration, Timestamp};

/// A state machine is a signal processor that maintains a state machine internally.
/// The state transition is defined as a lambda function passed in when construction.
/// The state transition is triggered when the control input gets changed.
/// The output is simply the current internal state.
#[derive(Serialize)]
pub struct StateMachine<Input, State, TransitionFunc, Trigger> {
    state: State,
    #[serde(skip_serializing)]
    transition: TransitionFunc,
    last_trigger_value: Trigger,
    _phantom: PhantomData<Input>,
}

impl<I, S, F, T> Debug for StateMachine<I, S, F, T>
where
    S: Debug + Clone,
    T: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StateMachine")
            .field("state", &self.state)
            .field("last_trigger_value", &self.last_trigger_value)
            .field("_phantom", &self._phantom)
            .finish()
    }
}

impl<I, S, F, T> StateMachine<I, S, F, T>
where
    S: Clone,
    T: Default,
{
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
    State: Clone + Serialize,
    Trigger: Clone + Eq + Serialize,
{
    type Input = (Trigger, Input);

    type Output = State;

    fn update(
        &mut self,
        _: &mut UpdateContext<EventIterator>,
        (trigger, input): &'a Self::Input,
    ) -> Self::Output {
        if trigger != &self.last_trigger_value {
            self.state = (self.transition)(&self.state, input);
            self.last_trigger_value = trigger.clone();
        }
        self.state.clone()
    }
}

#[derive(Serialize)]
pub struct SlidingWindow<Input, EmitFunc, Trigger, Output> {
    queue: VecDeque<Input>,
    #[serde(skip_serializing)]
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
    Output: Clone + Serialize,
    Trigger: Clone + Eq + Serialize,
    Input: Clone + Serialize,
{
    type Input = (Trigger, Input);

    type Output = Output;

    fn update(
        &mut self,
        _: &mut UpdateContext<Iter>,
        (trigger, input): &'a Self::Input,
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

#[derive(Serialize)]
pub struct SlidingTimeWindow<Input, EmitFunc, Trigger, Output> {
    queue: VecDeque<(Input, Timestamp)>,
    time_window_size: Duration,
    #[serde(skip_serializing)]
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
    Output: Clone + Serialize,
    Trigger: Clone + Eq + Serialize,
    Input: Clone + Serialize,
{
    type Input = (Trigger, Input);

    type Output = Output;

    fn update(
        &mut self,
        ctx: &mut UpdateContext<Iter>,
        (trigger, input): &'a Self::Input,
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
