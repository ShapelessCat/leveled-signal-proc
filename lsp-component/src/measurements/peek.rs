use lsp_runtime::{measurement::Measurement, UpdateContext, Timestamp};

#[derive(Default)]
pub struct Peek<T>(T);

impl <T : Clone, I: Iterator> Measurement<I> for Peek<T> {
    type Input = T;

    type Output = T;

    fn update(&mut self, _: &mut UpdateContext<I>, v: &Self::Input) {
        self.0 = v.clone();
    }

    fn measure_at(&self, _: &mut UpdateContext<I>, _: Timestamp) -> Self::Output {
        self.0.clone()
    }
}
