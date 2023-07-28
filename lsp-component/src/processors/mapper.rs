use std::marker::PhantomData;

use lsp_runtime::signal::SingnalProcessor;
use lsp_runtime::UpdateContext;

/// Mapping input signals statelessly to a output signal
pub struct SignalMapper<T, U, F> {
    how: F,
    _phantom_data: PhantomData<(T, U)>,
}

impl <T, U, F> SignalMapper<T, U, F>
where
    F: FnMut(&T) -> U
{
    pub fn new(how: F) -> Self {
        SignalMapper { how, _phantom_data: PhantomData }
    }
}

impl <'a, T: 'a, U, F, I:Iterator> SingnalProcessor<'a, I> for SignalMapper<T, U, F> 
where
    F: FnMut(&T) -> U
{
    type Input = &'a T;

    type Output = U;

    #[inline(always)]
    fn update(&mut self, _: &mut UpdateContext<I>, input: &T) -> U {
        (self.how)(input)
    }
}
