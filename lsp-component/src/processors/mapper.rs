use std::fmt::Debug;
use std::marker::PhantomData;

use serde::Serialize;

use lsp_runtime::context::UpdateContext;
use lsp_runtime::signal_api::{Patchable, SignalProcessor};

/// Mapping each input signal statelessly to an output signal.
#[derive(Serialize, Patchable)]
pub struct SignalMapper<ParamType, OutputType, ClosureType> {
    #[serde(skip)]
    how: ClosureType,
    #[serde(skip)]
    _phantom_data: PhantomData<(ParamType, OutputType)>,
}

impl<P, O, C> Debug for SignalMapper<P, O, C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SignalMapper").finish()
    }
}

impl<P, O, C: FnMut(&P) -> O> SignalMapper<P, O, C> {
    pub fn new(how: C) -> Self {
        SignalMapper {
            how,
            _phantom_data: PhantomData,
        }
    }
}

impl<'a, I, P, O, C> SignalProcessor<'a, I> for SignalMapper<P, O, C>
where
    I: Iterator,
    C: FnMut(&P) -> O,
{
    type Input = P;

    type Output = O;

    #[inline(always)]
    fn update(&mut self, _: &mut UpdateContext<I>, input: &'a Self::Input) -> O {
        (self.how)(input)
    }
}

// #[derive(Deserialize)]
// pub struct SignalMapperState;
//
// impl<P, O, C> Patchable for SignalMapper<P, O, C> {
//     type State = SignalMapperState;
//
//     fn patch(&mut self, _state: &str) {}
//
//     fn patch_from(&mut self, _state: Self::State) {}
// }

#[cfg(test)]
mod test {
    use lsp_runtime::signal_api::{Patchable, SignalProcessor};

    use super::SignalMapper;
    use crate::test::create_lsp_context_for_test;

    #[test]
    fn test_signal_mapper() {
        let mut mapper = SignalMapper::new(|x: &i32| x + 1);
        let mut ctx = create_lsp_context_for_test();
        let mut uc = ctx.borrow_update_context();
        assert_eq!(mapper.update(&mut uc, &1), 2);
        assert_eq!(mapper.update(&mut uc, &2), 3);

        let state = mapper.to_state();
        let mut init_mapper = SignalMapper::new(|x: &i32| x + 1);
        init_mapper.patch(&state);
        assert_eq!(state, init_mapper.to_state());
    }
}
