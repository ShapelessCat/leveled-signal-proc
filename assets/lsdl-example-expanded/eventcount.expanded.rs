// Recursive expansion of include_lsp_ir! macro
// =============================================

const _: () = {
    "";
};
#[derive(Clone, Default)]
pub struct InputSignalBag {
    pub user_action: String,
    pub user_action_clock: u64,
    pub page: String,
    pub page_clock: u64,
}
#[derive(serde::Deserialize, Clone)]
pub struct InputSignalBagPatch {
    #[serde(rename = "timestamp")]
    timestamp: chrono::DateTime<chrono::Utc>,
    #[serde(rename = "userAction")]
    pub user_action: Option<String>,
    #[serde(rename = "page")]
    pub page: Option<String>,
}
impl lsp_runtime::WithTimestamp for InputSignalBagPatch {
    fn timestamp(&self) -> lsp_runtime::Timestamp {
        self.timestamp.timestamp_nanos() as u64
    }
}
impl lsp_runtime::InputSignalBag for InputSignalBag {
    type Input = InputSignalBagPatch;
    fn patch(&mut self, patch: InputSignalBagPatch) {
        if let Some(value) = patch.user_action {
            self.user_action_clock += 1;
            self.user_action = value;
        }
        if let Some(value) = patch.page {
            self.page_clock += 1;
            self.page = value;
        }
    }
    fn should_measure(&mut self) -> bool {
        true
    }
}
#[derive(serde::Serialize)]
#[allow(non_snake_case)]
pub struct MetricsBag {
    pCount: i32,
}
pub fn lsp_main<InputIter, OutputHandler, Inst>(
    input_iter: InputIter,
    mut out_handle: OutputHandler,
    instrument_ctx: &mut Inst,
) -> Result<(), anyhow::Error>
where
    InputIter: Iterator<Item = InputSignalBagPatch>,
    OutputHandler: FnMut(MetricsBag) -> Result<(), anyhow::Error>,
    Inst: lsp_runtime::instrument::LspDataLogicInstrument,
{
    use lsp_runtime::LspContext;
    use serde_json::Deserializer;
    let mut input_state = Default::default();
    let mut __lsp_node_0 = {
        use lsp_component::processors::SignalMapper;
        SignalMapper::new(|lhs: &String| *lhs == "P")
    };
    let mut __lsp_output_buffer_0;
    let mut __lsp_node_1 = {
        use lsp_component::processors::Latch;
        Latch::<u64>::default()
    };
    let mut __lsp_output_buffer_1;
    let mut __lsp_node_2 = {
        use lsp_component::processors::Accumulator;
        Accumulator::<i32, _, _>::with_event_filter(Default::default(), |_| true)
    };
    let mut __lsp_output_buffer_2;
    let mut __lsp_node_3 = {
        use lsp_component::measurements::Peek;
        Peek::default()
    };
    let mut __lsp_output_buffer_3;
    let mut ctx = LspContext::<_, InputSignalBag>::new(input_iter);
    while let Some(moment) = ctx.next_event(&mut input_state) {
        instrument_ctx.data_logic_update_begin();
        let mut update_context = ctx.borrow_update_context();
        if moment.should_update_signals() {
            {
                use lsp_runtime::measurement::Measurement;
                use lsp_runtime::signal::SignalProcessor;
                instrument_ctx.node_update_begin(0usize);
                __lsp_output_buffer_0 =
                    __lsp_node_0.update(&mut update_context, &input_state.user_action);
                instrument_ctx.node_update_end(0usize);
                instrument_ctx.handle_node_output(&__lsp_output_buffer_0);
            }
            {
                use lsp_runtime::measurement::Measurement;
                use lsp_runtime::signal::SignalProcessor;
                instrument_ctx.node_update_begin(1usize);
                __lsp_output_buffer_1 = __lsp_node_1.update(
                    &mut update_context,
                    (&__lsp_output_buffer_0, &input_state.user_action_clock),
                );
                instrument_ctx.node_update_end(1usize);
                instrument_ctx.handle_node_output(&__lsp_output_buffer_1);
            }
            {
                use lsp_runtime::measurement::Measurement;
                use lsp_runtime::signal::SignalProcessor;
                instrument_ctx.node_update_begin(2usize);
                __lsp_output_buffer_2 = __lsp_node_2.update(
                    &mut update_context,
                    (&__lsp_output_buffer_1, &{
                        let _temp: i32 = 1i32;
                        _temp
                    }),
                );
                instrument_ctx.node_update_end(2usize);
                instrument_ctx.handle_node_output(&__lsp_output_buffer_2);
            }
            {
                use lsp_runtime::measurement::Measurement;
                use lsp_runtime::signal::SignalProcessor;
                instrument_ctx.node_update_begin(3usize);
                __lsp_output_buffer_3 =
                    __lsp_node_3.update(&mut update_context, &__lsp_output_buffer_2);
                instrument_ctx.node_update_end(3usize);
                instrument_ctx.handle_node_output(&__lsp_output_buffer_3);
            };
        }
        if moment.should_take_measurements() {
            let _metrics_bag = MetricsBag {
                pCount: {
                    use lsp_runtime::measurement::Measurement;
                    __lsp_node_3.measure(&mut update_context)
                },
            };
            instrument_ctx.data_logic_update_end();
            out_handle(_metrics_bag)?;
        } else {
            instrument_ctx.data_logic_update_end();
        }
    }
    Ok(())
}