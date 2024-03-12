use std::fmt::{Debug, Display};
use std::ops::Sub;

use serde::Serialize;

use lsp_runtime::{signal_api::SignalMeasurement, UpdateContext};

#[derive(Clone, Debug, Serialize)]
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

impl<'a, EventIterator, ScopeType, MeasurementType, Output> SignalMeasurement<'a, EventIterator>
    for ScopedMeasurement<ScopeType, MeasurementType, Output>
where
    EventIterator: Iterator,
    ScopeType: Serialize + Clone + Eq + Debug,
    Output: Serialize + Clone + Sub<Output = Output> + Display,
    MeasurementType: SignalMeasurement<'a, EventIterator, Output = Output>,
{
    type Input = (ScopeType, MeasurementType::Input);
    type Output = Output;

    fn update(&mut self, ctx: &mut UpdateContext<EventIterator>, (level, data): &'a Self::Input) {
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
