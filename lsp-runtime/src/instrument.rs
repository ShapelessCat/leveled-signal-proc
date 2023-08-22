use std::{time::Instant, fmt::Display};

pub trait NodeOutputHandler<'a, T> {
    fn new(value: &'a T) -> Self;
    fn handle_node_output<Instr: LspDataLogicInstrument + ?Sized>(&self, instrument_ctx: &mut Instr);
}

pub trait LspDataLogicInstrument : Display {
    type NodeOutputHandler<'a, T> : NodeOutputHandler<'a, T>;
    #[inline(always)]
    fn data_logic_update_begin(&mut self){}
    #[inline(always)]
    fn data_logic_update_end(&mut self){}
    #[inline(always)]
    fn node_update_begin(&mut self, _node_id: usize){}
    #[inline(always)]
    fn node_update_end(&mut self, _node_id: usize){}
    #[inline(always)]
    fn handle_node_output<'a, T>(&mut self, node_output: &'a T) {
        let wrapped = <Self::NodeOutputHandler<'a, T> as NodeOutputHandler<'a, T>>::new(node_output);
        wrapped.handle_node_output(self);
    }
}

pub struct DropNodeOutput;
impl <'a, T> NodeOutputHandler<'a, T> for DropNodeOutput {
    #[inline(always)]
    fn new(_: &'a T) -> Self {
        Self
    }
    #[inline(always)]
    fn handle_node_output<Instr: LspDataLogicInstrument + ?Sized>(&self, _: &mut Instr) {}
}

#[derive(Default)]
pub struct NoInstrument;
impl Display for NoInstrument {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
       Ok(()) 
    }
}
impl LspDataLogicInstrument for NoInstrument {
    type NodeOutputHandler<'a, T> = DropNodeOutput;
}

#[derive(Default)]
pub struct InstrumentDataLogicRunningTime{
    pub data_logic_running_time_secs: f64,
    data_logic_update_start_timestamp: Option<Instant>,
}

impl Display for InstrumentDataLogicRunningTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DataLogicRunningTimeInSecs = {}", self.data_logic_running_time_secs)
    }
}

impl LspDataLogicInstrument for InstrumentDataLogicRunningTime {
    type NodeOutputHandler<'a, T> = DropNodeOutput;

    #[inline(always)]
    fn data_logic_update_begin(&mut self){
        self.data_logic_update_start_timestamp = Some(Instant::now());
    }

    #[inline(always)]
    fn data_logic_update_end(&mut self){
        self.data_logic_update_start_timestamp.take().map(|start_ts| {
            let end_ts = Instant::now();
            self.data_logic_running_time_secs += end_ts.duration_since(start_ts).as_secs_f64();
        });
    }
}
