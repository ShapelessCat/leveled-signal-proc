use std::marker::PhantomData;

use lsp_runtime::signal::SignalProcessor;
use lsp_runtime::UpdateContext;

/// Mapping input signals statelessly to a output signal
pub struct SignalMapper<ParamType, OutputType, ClosureType> {
    how: ClosureType,
    _phantom_data: PhantomData<(ParamType, OutputType)>,
}

impl<T, U, F> SignalMapper<T, U, F>
where
    F: FnMut(&T) -> U,
{
    pub fn new(how: F) -> Self {
        SignalMapper {
            how,
            _phantom_data: PhantomData,
        }
    }
}

impl<'a, T: 'a, U, F, I: Iterator> SignalProcessor<'a, I> for SignalMapper<T, U, F>
where
    F: FnMut(&T) -> U,
{
    type Input = &'a T;

    type Output = U;

    #[inline(always)]
    fn update(&mut self, _: &mut UpdateContext<I>, input: &T) -> U {
        (self.how)(input)
    }
}

#[cfg(test)]
mod test {
    use lsp_runtime::signal::SignalProcessor;

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