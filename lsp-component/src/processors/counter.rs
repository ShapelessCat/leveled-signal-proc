use lsp_runtime::{signal::SingnalProcessor, UpdateContext};


/// A ValueChangeCounter counts the number of changes in the input 
#[derive(Default)]
pub struct ValueChangeCounter<T:Clone + Eq> {
    prev: Option<T>,
    counter: usize,
}

impl <T: Clone + Eq> ValueChangeCounter<T> {
    pub fn with_init_value(value: T) -> Self {
        Self {
            prev: Some(value),
            counter: 0,
        }
    }
}

impl <T: Clone + Eq, I: Iterator> SingnalProcessor<I> for ValueChangeCounter<T> {
    type Input = T;

    type Output = usize;

    #[inline(always)]
    fn update(&mut self, _: &mut UpdateContext<I>, input: &Self::Input) -> Self::Output {
        if self.prev.as_ref().map_or(true, |value| value != input) {
            self.counter += 1;
            self.prev = Some(input.clone());
        }
        self.counter
    }
}
