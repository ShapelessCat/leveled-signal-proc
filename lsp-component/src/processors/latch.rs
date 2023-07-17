use lsp_runtime::signal::SingnalProcessor;
use lsp_runtime::UpdateContext;

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
