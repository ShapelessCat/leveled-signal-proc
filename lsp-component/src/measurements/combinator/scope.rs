use std::fmt::{Debug, Display};
use std::ops::Sub;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use lsp_runtime::context::UpdateContext;
use lsp_runtime::signal_api::{Patchable, SignalMeasurement};

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
    ScopeType: Clone + Eq + Debug,
    Output: Clone + Sub<Output = Output> + Display,
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

#[derive(Deserialize)]
pub struct ScopedMeasurementState<ScopeType, MeasurementStateType, MeasurementOutput> {
    current_control_level: ScopeType,
    inner: MeasurementStateType,
    current_base: MeasurementOutput,
}

impl<S, M, O> Patchable for ScopedMeasurement<S, M, O>
where
    S: Serialize + DeserializeOwned,
    M: Serialize + Patchable,
    O: Serialize + DeserializeOwned,
{
    type State = ScopedMeasurementState<S, M::State, O>;

    fn patch_from(&mut self, state: Self::State) {
        self.current_control_level = state.current_control_level;
        self.inner.patch_from(state.inner);
        self.current_base = state.current_base;
    }
}
