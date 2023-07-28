use lsp_runtime::{measurement::Measurement, UpdateContext};

#[derive(Default)]
pub struct Peek<T>(T);

impl <'a, T : Clone + 'a, I: Iterator> Measurement<'a, I> for Peek<T> {
    type Input = &'a T;

    type Output = T;

    fn update(&mut self, _: &mut UpdateContext<I>, v: Self::Input) {
        self.0 = v.clone();
    }

    fn measure(&self, _: &mut UpdateContext<I>) -> Self::Output {
        self.0.clone()
    }
}
