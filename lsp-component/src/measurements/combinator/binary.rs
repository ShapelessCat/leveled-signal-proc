use std::marker::PhantomData;

use lsp_runtime::{measurement::Measurement, UpdateContext};

#[derive(Clone, Debug)]
pub struct BinaryCombinedMeasurement<OutputType0, OutputType1, OutputType, ClosureType, MeasurementType0, MeasurementType1> {
    binary_op: ClosureType,
    inner0: MeasurementType0,
    inner1: MeasurementType1,
    _phantom_data: PhantomData<(OutputType0, OutputType1, OutputType)>,
}

impl<OutputType0, OutputType1, OutputType, ClosureType, MeasurementType0, MeasurementType1>
    BinaryCombinedMeasurement<OutputType0, OutputType1, OutputType, ClosureType, MeasurementType0, MeasurementType1>
where
    ClosureType: Fn(&OutputType0, &OutputType1) -> OutputType,
{
    pub fn new(binary_op: ClosureType, inner0: MeasurementType0, inner1: MeasurementType1) -> Self {
        BinaryCombinedMeasurement {
            binary_op,
            inner0,
            inner1,
            _phantom_data: PhantomData,
        }
    }
}

impl<'a, EventIterator, OutputType0, OutputType1, OutputType, ClosureType, MeasurementType0, MeasurementType1>
    Measurement<'a, EventIterator>
    for BinaryCombinedMeasurement<OutputType0, OutputType1, OutputType, ClosureType, MeasurementType0, MeasurementType1>
where
    EventIterator: Iterator,
    OutputType: Clone + std::fmt::Display,
    ClosureType: Fn(&OutputType0, &OutputType1) -> OutputType,
    MeasurementType0: Measurement<'a, EventIterator, Output = OutputType0>,
    MeasurementType1: Measurement<'a, EventIterator, Output = OutputType1>,
{
    type Input = (MeasurementType0::Input, MeasurementType1::Input);
    type Output = OutputType;

    fn update(&mut self, ctx: &mut UpdateContext<EventIterator>, (input0, input1): Self::Input) {
        self.inner0.update(ctx, input0);
        self.inner1.update(ctx, input1);
    }

    fn measure(&self, ctx: &mut UpdateContext<EventIterator>) -> Self::Output {
        (self.binary_op)(&self.inner0.measure(ctx), &self.inner1.measure(ctx))
    }
}
