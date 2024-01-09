use std::ops::Sub;

use lsp_runtime::{measurement::Measurement, UpdateContext};

#[derive(Clone, Debug)]
pub struct ScopedMeasurement<ScopeType, MeasurementType, MeasurementOutput> {
    current_control_level: ScopeType,
    inner: MeasurementType,
    current_base: MeasurementOutput,
}

impl<ScopeType, MeasurementType, MeasurementOutput>
    ScopedMeasurement<ScopeType, MeasurementType, MeasurementOutput>
where
    ScopeType: Default,
    MeasurementOutput: Default,
{
    pub fn new(inner: MeasurementType) -> Self {
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
    EventIterator: Iterator,
    ScopeType: Clone + Eq + 'a + std::fmt::Debug,
    Output: Clone + Sub<Output = Output> + std::fmt::Display,
    MeasurementType: Measurement<'a, EventIterator, Output = Output>,
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
