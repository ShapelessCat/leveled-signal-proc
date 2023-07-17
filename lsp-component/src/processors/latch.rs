use lsp_runtime::signal::SingnalProcessor;
use lsp_runtime::UpdateContext;

/// A latch is a signal processor that takes a control input and a data input.
/// For each time, a latch produce the same output as the internal state.
/// When the control input becomes true, the latch change it internal state to the data input.
/// This concept borrowed from the hardware component which shares the same name. And it's widely use
/// as one bit memory in digital circuits. 
#[derive(Default)]
pub struct Latch<T: Clone>(T);

impl <T: Clone, I:Iterator> SingnalProcessor<I> for Latch<T> {
    type Input = (bool, T);
    type Output = T;
    #[inline(always)]
    fn update(&mut self, _: &mut UpdateContext<I>, &(ref set, ref value): &Self::Input) -> T {
        if *set {
            self.0 = value.clone();
        }
        self.0.clone()
    }
}
