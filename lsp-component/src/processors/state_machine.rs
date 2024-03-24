use std::fmt::Debug;
use std::marker::PhantomData;

use serde::Serialize;

use lsp_runtime::context::UpdateContext;
use lsp_runtime::signal_api::SignalProcessor;

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
