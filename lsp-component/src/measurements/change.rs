use std::ops::Sub;

use lsp_runtime::{measurement::Measurement, UpdateContext};

#[derive(Default)]
pub struct ChangeSinceCurrentLevel<ControlSignal, DataSignal> {
    current_control_level: ControlSignal,
    data_at_current_level_starts: DataSignal,
    data_at_frontier: DataSignal,
}

impl<'a, ControlSignal, DataSignal, EvIt> Measurement<'a, EvIt>
    for ChangeSinceCurrentLevel<ControlSignal, DataSignal>
where
    ControlSignal: PartialEq + Clone + 'a,
    DataSignal: Sub<DataSignal> + Clone + 'a,
    EvIt: Iterator,
{
    type Input = (&'a ControlSignal, &'a DataSignal);
    type Output = DataSignal::Output;
    fn update(&mut self, _: &mut UpdateContext<EvIt>, (cont, data): Self::Input) {
        if cont != &self.current_control_level {
            self.current_control_level = cont.clone();
            self.data_at_current_level_starts = data.clone();
        }
        self.data_at_frontier = data.clone();
    }
    fn measure(&self, _: &mut UpdateContext<EvIt>) -> Self::Output {
        self.data_at_frontier.clone() - self.data_at_current_level_starts.clone()
    }
}