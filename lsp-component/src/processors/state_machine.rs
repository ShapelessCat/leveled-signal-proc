use std::marker::PhantomData;

use lsp_runtime::{signal::SingnalProcessor, UpdateContext};


pub struct StateMachine<I, S: Clone, F, T> {
    state: S,
    transition: F,
    last_trigger_value: T,
    _phantom:PhantomData<I>,
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

impl <Input, State, Transition, Iter, Trigger> SingnalProcessor<Iter> for StateMachine<Input, State, Transition, Trigger>
where
    Transition: Fn(&State, &Input) -> State,
    Iter:Iterator,
    State: Clone,
    Trigger: Eq + Clone,
{
    type Input = (Trigger, Input);

    type Output = State;

    fn update(&mut self, _: &mut UpdateContext<Iter>, &(ref trigger, ref input): &Self::Input) -> Self::Output {
        if trigger != &self.last_trigger_value {
            self.state = (self.transition)(&self.state, input);
            self.last_trigger_value = trigger.clone();
        }
        self.state.clone()
    }
}
