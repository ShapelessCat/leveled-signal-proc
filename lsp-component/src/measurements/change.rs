use std::ops::Sub;

use lsp_runtime::{measurement::Measurement, UpdateContext, Timestamp};



#[derive(Default)]
pub struct ChangeSinceCurrentLevel<ControlSignal, DataSignal> {
    current_control_level: ControlSignal,
    data_at_current_level_starts: DataSignal,
    data_at_frontier: DataSignal,
}

impl <ControlSignal, DataSignal, EvIt> Measurement<EvIt> for ChangeSinceCurrentLevel<ControlSignal, DataSignal> 
where
    ControlSignal: PartialEq + Clone,
    DataSignal: Sub<DataSignal> + Clone,
    EvIt: Iterator
{
    type Input = (ControlSignal, DataSignal);
    type Output = DataSignal::Output;
    fn update(&mut self, _: &mut UpdateContext<EvIt>, &(ref cont,ref data): &Self::Input) {
        if cont != &self.current_control_level {
            self.current_control_level = cont.clone();
            self.data_at_current_level_starts = data.clone();
        }
        self.data_at_frontier = data.clone();
    }
    fn measure_at(&self, _: &mut UpdateContext<EvIt>, _: Timestamp) -> Self::Output {
        self.data_at_frontier.clone() - self.data_at_current_level_starts.clone()
    }
}