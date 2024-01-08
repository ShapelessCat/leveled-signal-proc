use std::marker::PhantomData;

use lsp_runtime::{measurement::Measurement, UpdateContext};

#[derive(Clone, Default, Debug)]
pub struct MappedMeasurement<InnerOutput, OutputType, ClosureType, MeasurementType> {
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
    Measurement<'a, EventIterator>
    for MappedMeasurement<InnerOutput, OutputType, ClosureType, MeasurementType>
where
    MeasurementType: Measurement<'a, EventIterator, Output = InnerOutput>,
    InnerOutput: Clone + std::fmt::Display,
    ClosureType: Fn(&InnerOutput) -> OutputType,
    EventIterator: Iterator,
{
    type Input = MeasurementType::Input;
    type Output = OutputType;

    fn update(&mut self, ctx: &mut UpdateContext<EventIterator>, input: Self::Input) {
        self.inner.update(ctx, input)
    }

    fn measure(&self, ctx: &mut UpdateContext<EventIterator>) -> Self::Output {
        (self.how)(&self.inner.measure(ctx))
    }
}

pub trait MappedMeasurementExt<'a, I: Iterator>: Measurement<'a, I> + Sized
where
    Self::Output: Clone,
{
    fn mapped<OutputType, ClosureType>(
        self,
        how: ClosureType,
    ) -> MappedMeasurement<Self::Output, OutputType, ClosureType, Self>
    where
        ClosureType: Fn(&Self::Output) -> OutputType,
    {
        MappedMeasurement {
            inner: self,
            how,
            _phantom_data: PhantomData,
        }
    }
}

impl<'a, I, M> MappedMeasurementExt<'a, I> for M
where
    M: Measurement<'a, I> + Sized,
    M::Output: Clone,
    I: Iterator,
{
}
