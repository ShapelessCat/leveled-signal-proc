use std::fmt::Debug;
use std::marker::PhantomData;

use serde::Serialize;

use lsp_runtime::context::UpdateContext;
use lsp_runtime::signal_api::{Patchable, SignalProcessor};

/// A state machine is a signal processor that maintains a state machine internally.
///
/// The state transition is defined as a lambda function passed in when construction.
/// The state transition is triggered when the control input gets changed.
/// The output is simply the current internal state.
#[derive(Serialize, Patchable)]
pub struct StateMachine<Input, State, TransitionFunc, Trigger> {
    state: State,
    #[serde(skip)]
    transition: TransitionFunc,
    last_trigger_value: Trigger,
    #[serde(skip)]
    _phantom_data: PhantomData<Input>,
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
            _phantom_data: PhantomData,
        }
    }
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
            .field("_phantom_data", &self._phantom_data)
            .finish()
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

// #[derive(Deserialize)]
// pub struct StateMachineState<State, Trigger> {
//     state: State,
//     last_trigger_value: Trigger,
// }
//
// impl<I, S, F, T> Patchable for StateMachine<I, S, F, T>
// where
//     S: Serialize + DeserializeOwned,
//     T: Serialize + DeserializeOwned,
// {
//     type State = StateMachineState<S, T>;
//
//     fn patch_from(&mut self, state: Self::State) {
//         self.state = state.state;
//         self.last_trigger_value = state.last_trigger_value;
//     }
// }
