use std::ops::Sub;

use lsp_runtime::{measurement::Measurement, UpdateContext};

#[derive(Clone, Default, Debug)]
pub struct ScopedMeasurement<ScopeType, Measurement, MeasurementOutput> {
    current_control_level: ScopeType,
    inner: Measurement,
    current_base: MeasurementOutput,
}

impl<ScopeType, Measurement, MeasurementOutput>
    ScopedMeasurement<ScopeType, Measurement, MeasurementOutput>
where
    ScopeType: Default,
    MeasurementOutput: Default,
{
    pub fn new(inner: Measurement) -> Self {
        ScopedMeasurement {
            current_control_level: ScopeType::default(),
            inner,
            current_base: MeasurementOutput::default(),
        }
    }
}

impl<'a, EventIterator, ScopeType, MeasurementType, Output> Measurement<'a, EventIterator>
    for ScopedMeasurement<ScopeType, MeasurementType, Output>
where
    MeasurementType: Measurement<'a, EventIterator, Output = Output>,
    Output: Clone + Sub<Output = Output> + std::fmt::Display,
    ScopeType: Clone + Eq + 'a + std::fmt::Debug,
    EventIterator: Iterator,
{
    type Input = (&'a ScopeType, MeasurementType::Input);
    type Output = Output;

    fn update(&mut self, ctx: &mut UpdateContext<EventIterator>, (level, data): Self::Input) {
        if &self.current_control_level != level {
            self.current_base = self.inner.measure(ctx);
            self.current_control_level = level.clone();
        }

        self.inner.update(ctx, data);
    }

    fn measure(&self, ctx: &mut UpdateContext<EventIterator>) -> Self::Output {
        let base = self.current_base.clone();
        self.inner.measure(ctx) - base
    }
}

pub trait ScopedMeasurementExt<'a, I: Iterator>: Measurement<'a, I> + Sized
where
    Self::Output: Clone + Sub<Output = Self::Output> + Default,
{
    fn scoped<ControlType: Default>(
        self,
        initial_level: ControlType,
    ) -> ScopedMeasurement<ControlType, Self, Self::Output> {
        ScopedMeasurement {
            current_control_level: initial_level,
            inner: self,
            current_base: Default::default(),
        }
    }
}

impl<'a, I, M> ScopedMeasurementExt<'a, I> for M
where
    M: Measurement<'a, I> + Sized,
    M::Output: Clone + Sub<Output = M::Output> + Default,
    I: Iterator,
{
}
