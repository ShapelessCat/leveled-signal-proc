use std::marker::PhantomData;

use crate::UpdateContext;

pub trait SingnalProcessor<'a, EventIt: Iterator> {
    type Input : 'a;
    type Output;

    /// Update the signal - the data readiness contraint requires the output must be valid.
    /// The semantics of this method is follow: All the input signals are defined by parameter `input` from now. 
    /// And the output is also valid from the now.
    /// Data readiness isn't a problem in most of the computed signals. 
    fn update(&mut self, ctx: &mut UpdateContext<EventIt>, input: Self::Input) -> Self::Output;

    fn map_input<MapClosure, From, To>(self, closure: MapClosure) -> InputMap<MapClosure, From, To, Self> 
    where
        MapClosure: FnMut(From) -> To,
        Self: Sized
    {
        InputMap { input_map_closure: closure, downstream: self, _phantom: PhantomData }
    }
}

pub struct InputMap<Closure, MapIn, MapOut, InnerSignalProc> {
    input_map_closure: InputMap,
    downstream: InnerSignalProc,
    _phantom: PhantomData<(MapIn, MapOut)>
}

impl <'a, Eit, Map, MapIn, MapOut, SigProc> SingnalProcessor<'a, Eit> for InputMap<Map, MapIn, MapOut, SigProc>
where
    Eit: Iterator,
    Map: FnMut(MapIn) -> MapOut,
    MapIn: 'a,
    SigProc: SingnalProcessor<'a, Eit, Input = MapOut>,
{
    type Input = MapIn;

    type Output = SigProc::Output;

    fn update(&mut self, ctx: &mut UpdateContext<Eit>, input: Self::Input) -> Self::Output {
        self.downstream.update(ctx, (self.input_map_closure)(input))
    }
}