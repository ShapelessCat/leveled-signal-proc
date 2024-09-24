use std::collections::VecDeque;

use std::marker::PhantomData;

use serde::Serialize;

use lsp_runtime::context::UpdateContext;
use lsp_runtime::signal_api::{Patchable, SignalProcessor};
use lsp_runtime::{Duration, Timestamp};

/// TODO: !!!
/// TODO: `SlidingWindow` and `SlidingTimeWindow` are not fundamental, try to implement these two
///       processors with `StateMachine`.
#[derive(Serialize, Patchable)]
pub struct SlidingWindow<Input, EmitFunc, Trigger, Output> {
    queue: VecDeque<Input>,
    #[serde(skip)]
    emit_func: EmitFunc,
    last_trigger_value: Trigger,
    last_dequeued_value: Input,
    #[serde(skip)]
    _phantom_data: PhantomData<Output>,
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
            _phantom_data: PhantomData,
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

// #[derive(Deserialize)]
// pub struct SlidingWindowState<Input, Trigger> {
//     queue: VecDeque<Input>,
//     last_trigger_value: Trigger,
//     last_dequeued_value: Input,
// }
//
// impl<I, E, T, O> Patchable for SlidingWindow<I, E, T, O>
// where
//     I: Serialize + DeserializeOwned,
//     T: Serialize + DeserializeOwned,
// {
//     type State = SlidingWindowState<I, T>;
//
//     fn patch_from(&mut self, state: Self::State) {
//         self.queue = state.queue;
//         self.last_trigger_value = state.last_trigger_value;
//         self.last_dequeued_value = state.last_dequeued_value;
//     }
// }

#[derive(Serialize, Patchable)]
pub struct SlidingTimeWindow<Input, EmitFunc, Trigger, Output> {
    queue: VecDeque<(Input, Timestamp)>,
    time_window_size: Duration,
    #[serde(skip)]
    emit_func: EmitFunc,
    last_trigger_value: Trigger,
    last_dequeued_value: Input,
    #[serde(skip)]
    _phantom_data: PhantomData<Output>,
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
            _phantom_data: PhantomData,
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

// #[derive(Deserialize)]
// pub struct SlidingTimeWindowState<Input, Trigger> {
//     queue: VecDeque<(Input, Timestamp)>,
//     time_window_size: Duration,
//     last_trigger_value: Trigger,
//     last_dequeued_value: Input,
// }
//
// impl<I, E, T, O> Patchable for SlidingTimeWindow<I, E, T, O>
// where
//     I: Serialize + DeserializeOwned,
//     T: Serialize + DeserializeOwned,
// {
//     type State = SlidingTimeWindowState<I, T>;
//
//     fn patch_from(&mut self, state: Self::State) {
//         self.queue = state.queue;
//         self.time_window_size = state.time_window_size;
//         self.last_trigger_value = state.last_trigger_value;
//         self.last_dequeued_value = state.last_dequeued_value;
//     }
// }
