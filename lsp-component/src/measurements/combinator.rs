use std::ops::Sub;

use lsp_runtime::{measurement::Measurement, UpdateContext, Timestamp};

#[derive(Default)]
pub struct ScopedMeasurement<ScopeType, Measurement, MeasurementOutput> {
    current_control_level: ScopeType,
    current_level_timestamp: Timestamp,
    inner: Measurement,
    current_base: MeasurementOutput,
    prev_base: Option<(Timestamp, MeasurementOutput)>,
}

impl <'a, EventIterator, ScopeType, MeasurementType, Output> Measurement<'a, EventIterator> for ScopedMeasurement<ScopeType, MeasurementType, Output>
where
    MeasurementType: Measurement<'a, EventIterator, Output = Output>,
    Output: Clone + Sub<Output = Output>,
    ScopeType: Clone + Eq + 'a,
    EventIterator: Iterator,
{
    type Input = (&'a ScopeType, MeasurementType::Input);
    type Output = Output;

    fn update(&mut self, ctx: &mut UpdateContext<EventIterator>, (level, data): Self::Input) {
        self.inner.update(ctx, data);

        if &self.current_control_level != level {
            self.current_control_level = level.clone();
            self.current_level_timestamp = ctx.frontier();
            self.prev_base = Some((ctx.frontier(), self.current_base.clone()));
            self.current_base = self.inner.measure(ctx);
        }
    }

    fn measure(&self, ctx: &mut UpdateContext<EventIterator>) -> Self::Output {
        let base = match &self.prev_base {
            Some((prev_ts, prev_base)) if *prev_ts == ctx.frontier() => prev_base.clone(),
            _ => self.current_base.clone(),
        };
        self.inner.measure(ctx) - base
    }
}

pub trait ScopedMeasurementExt<'a, I:Iterator> : Measurement <'a, I> + Sized
where 
    Self::Output: Clone + Sub<Output = Self::Output> + Default,
{
    fn scoped<ControlType: Default>(self, initial_level: ControlType) -> ScopedMeasurement<ControlType, Self, Self::Output>
    {
        ScopedMeasurement {
            current_control_level: initial_level,
            current_level_timestamp: Default::default(),
            inner: self,
            current_base: Default::default(),
            prev_base: None,
        }
    }
}

impl <'a, I, M> ScopedMeasurementExt<'a, I> for M 
where
    M: Measurement<'a, I> + Sized,
    M::Output: Clone + Sub<Output = M::Output> + Default,
    I:Iterator,
{}