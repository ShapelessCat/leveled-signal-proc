use std::fmt::Display;
use std::marker::PhantomData;

use serde::Serialize;

use lsp_runtime::context::UpdateContext;
use lsp_runtime::signal_api::SignalMeasurement;

#[derive(Clone, Debug, Serialize)]
pub struct MappedMeasurement<InnerOutput, OutputType, ClosureType, MeasurementType> {
    #[serde(skip_serializing)]
    how: ClosureType,
    inner: MeasurementType,
    _phantom_data: PhantomData<(InnerOutput, OutputType)>,
}

impl<InnerOutput, OutputType, ClosureType, MeasurementType>
    MappedMeasurement<InnerOutput, OutputType, ClosureType, MeasurementType>
where
    ClosureType: Fn(&InnerOutput) -> OutputType,
{
    pub fn new(how: ClosureType, inner: MeasurementType) -> Self {
        MappedMeasurement {
            how,
            inner,
            _phantom_data: PhantomData,
        }
    }
}

impl<'a, EventIterator, InnerOutput, OutputType, ClosureType, MeasurementType>
    SignalMeasurement<'a, EventIterator>
    for MappedMeasurement<InnerOutput, OutputType, ClosureType, MeasurementType>
where
    EventIterator: Iterator,
    InnerOutput: Clone + Display,
    ClosureType: Fn(&InnerOutput) -> OutputType,
    MeasurementType: SignalMeasurement<'a, EventIterator, Output = InnerOutput>,
{
    type Input = MeasurementType::Input;
    type Output = OutputType;

    fn update(&mut self, ctx: &mut UpdateContext<EventIterator>, input: &'a Self::Input) {
        self.inner.update(ctx, input)
    }

    fn measure(&self, ctx: &mut UpdateContext<EventIterator>) -> Self::Output {
        (self.how)(&self.inner.measure(ctx))
    }
}