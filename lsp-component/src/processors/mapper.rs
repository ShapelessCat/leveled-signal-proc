use std::fmt::Debug;
use std::marker::PhantomData;

use serde::{Deserialize, Serialize};

use lsp_runtime::context::UpdateContext;
use lsp_runtime::signal_api::{Patchable, SignalProcessor};

/// Mapping each input signal statelessly to an output signal.
#[derive(Serialize)]
pub struct SignalMapper<ParamType, OutputType, ClosureType> {
    #[serde(skip_serializing)]
    how: ClosureType,
    #[serde(skip_serializing)]
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

#[derive(Deserialize)]
struct SignalMapperState;

impl<P, O, C> Patchable for SignalMapper<P, O, C> {
    fn patch(&mut self, _state: &str) {}
}

#[cfg(test)]
mod test {
    use lsp_runtime::signal_api::SignalProcessor;

    use crate::{processors::SignalMapper, test::create_lsp_context_for_test};

    #[test]
    fn test_signal_mapper() {
        let mut mapper = SignalMapper::new(|x: &i32| x + 1);
        let mut ctx = create_lsp_context_for_test();
        let mut uc = ctx.borrow_update_context();
        assert_eq!(mapper.update(&mut uc, &1), 2);
        assert_eq!(mapper.update(&mut uc, &2), 3);
    }
}
