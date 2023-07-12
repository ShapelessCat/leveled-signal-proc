
use std::{error::Error, marker::PhantomData};

type TimeStamp = u64;

pub trait SignalProcessor {
    type Input;
    type Output;
    type Error : Error;
    fn reset(&mut self) -> Result<(), Self::Error>{
        Ok(())
    }
    fn apply(&mut self, ts: TimeStamp, input: &Self::Input) -> Result<Option<Self::Output>, Self::Error>;
}

pub struct TimeShift<T: Sized>(Option<T>);

impl <T: Sized + Default> Default for TimeShift<T> {
    fn default() -> Self {
        TimeShift(None)
    }
}

impl <T: Sized + Clone> SignalProcessor for TimeShift<T> {
    type Input = T;

    type Output = T;

    type Error = std::io::Error;

    fn reset(&mut self) -> Result<(), Self::Error> {
        self.0 = None;
        Ok(())
    }

    fn apply(&mut self, _: TimeStamp, input: &Self::Input) -> Result<Option<Self::Output>, Self::Error> {
        let ret = self.0.clone();
        self.0 = Some(input.clone());
        Ok(ret)
    }
}

pub struct MapSignal<T, U, F: Fn(TimeStamp, &T) -> U>(F, PhantomData<(T, U)>);

impl <T, U, F> MapSignal<T, U, F>
where
    F: Fn(TimeStamp, &T) -> U 
{
    pub fn new(f: F) -> Self {
        MapSignal(f, PhantomData::default())
    }
}

impl <T, U, F> SignalProcessor for MapSignal<T, U, F>
where
    F: Fn(TimeStamp, &T) -> U 
{
    type Input = T;

    type Output = U;

    type Error = std::io::Error;

    fn apply(&mut self, ts: TimeStamp, input: &Self::Input) -> Result<Option<Self::Output>, Self::Error> {
        Ok(Some((self.0)(ts, input)))
    }
}

#[derive(Default)]
pub struct Integral {
    start: TimeStamp,
    value: f64,
    output: f64,
}

impl SignalProcessor for Integral {
    type Input = f64;
    type Output = f64;
    type Error = std::io::Error;

    fn reset(&mut self) -> Result<(), Self::Error> {
        self.output = 0.0;
        Ok(())
    }

    fn apply(&mut self, ts: TimeStamp, input: &Self::Input) -> Result<Option<Self::Output>, Self::Error> {
        self.output += (ts - self.start) as f64 * self.value;
        self.start = ts;
        self.value = *input;
        Ok(Some(self.output))
    }
}
