use lsp_runtime::{Timestamp, signal::SingnalProcessor, UpdateContext};


/// 
/// Note: Although the duration of current level can not be a measurement, as it's a function of time
/// But duration of previous level is a well defined signal - duration of previous level is a known value
#[derive(Default)]
pub struct DurationOfPreviousLevel<T> {
    current_value: T,
    current_value_since: Timestamp,
    output_buf: Timestamp,
}

impl <'a, T: PartialEq + Clone + 'a, I:Iterator> SingnalProcessor<'a, I> for DurationOfPreviousLevel<T> {
    type Input = &'a T;

    type Output = Timestamp;

    fn update(&mut self, ctx: &mut UpdateContext<I>, input: Self::Input) -> Self::Output {
        if &self.current_value != input {
            self.output_buf = ctx.frontier() - self.current_value_since;
            self.current_value = input.clone();
            self.current_value_since = ctx.frontier();
        } 
        self.output_buf
    }
}