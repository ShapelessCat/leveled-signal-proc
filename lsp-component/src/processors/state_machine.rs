use std::marker::PhantomData;

use lsp_runtime::{signal::SingnalProcessor, UpdateContext};

/// A state machine is a signal processor that maintains a state machine internally.
/// The state transition is defined as a lambda function passed in when construction. 
/// The state transition is triggered when the control input gets changed.
/// The output is simply the current internal state
pub struct StateMachine<Input, State: Clone, TransitionFunc, Trigger> {
    state: State,
    transition: TransitionFunc,
    last_trigger_value: Trigger,
    _phantom:PhantomData<Input>,
}

impl <I, S: Clone, F, T: Default> StateMachine<I, S, F, T> {
    pub fn new(inital_state: S, transition: F) -> Self
    where
        F: Fn(&S, &I) -> S
    {
        Self {
            state: inital_state,
            transition,
            last_trigger_value: Default::default(),
            _phantom: PhantomData,
        }
    }
}

impl <'a, Input, State, Transition, Iter, Trigger> SingnalProcessor<'a, Iter> for StateMachine<Input, State, Transition, Trigger>
where
    Transition: Fn(&State, &Input) -> State,
    Iter:Iterator,
    State: Clone,
    Trigger: Eq + Clone + 'a,
    Input: 'a,
{
    type Input = (&'a Trigger, &'a Input);

    type Output = State;

    fn update(&mut self, _: &mut UpdateContext<Iter>, (trigger, input): Self::Input) -> Self::Output {
        if trigger != &self.last_trigger_value {
            self.state = (self.transition)(&self.state, input);
            self.last_trigger_value = trigger.clone();
        }
        self.state.clone()
    }
}
