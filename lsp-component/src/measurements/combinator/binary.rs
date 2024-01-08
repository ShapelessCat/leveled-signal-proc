use std::marker::PhantomData;

use lsp_runtime::{measurement::Measurement, UpdateContext};

#[derive(Default, Debug)]
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
    MeasurementType0: Measurement<'a, EventIterator, Output = OutputType0>,
    MeasurementType1: Measurement<'a, EventIterator, Output = OutputType1>,
    OutputType: Clone + std::fmt::Display,
    ClosureType: Fn(&OutputType0, &OutputType1) -> OutputType,
    EventIterator: Iterator,
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

pub trait BinaryCombinedMeasurementExt<'a, I: Iterator, MeasurementType1>: Measurement<'a, I> + Sized
where
    Self::Output: Clone,
    MeasurementType1: Measurement<'a, I>,
{
    fn combined<InputType1, OutputType1, OutputType, ClosureType>(
        self,
        binary_op: ClosureType,
        other: MeasurementType1,
    ) -> BinaryCombinedMeasurement<Self::Output, MeasurementType1::Output, OutputType, ClosureType, Self, MeasurementType1>
    where
        ClosureType: Fn(&Self::Output, OutputType1) -> OutputType,
    {
        BinaryCombinedMeasurement {
            binary_op,
            inner0: self,
            inner1: other,
            _phantom_data: PhantomData,
        }
    }
}

impl<'a, I, M, M1> BinaryCombinedMeasurementExt<'a, I, M1> for M
where
    M: Measurement<'a, I> + Sized,
    M1: Measurement<'a, I> + Sized,
    M::Output: Clone,
    M1::Output: Clone,
    I: Iterator,
{
}
