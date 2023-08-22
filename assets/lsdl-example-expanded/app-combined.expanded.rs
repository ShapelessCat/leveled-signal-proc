// Recursive expansion of include_lsp_ir! macro
// =============================================

const _: () = {
    "";
};
#[derive(Clone, Default)]
pub struct InputSignalBag {
    pub video_event: String,
    pub video_event_clock: u64,
    pub raw_event: String,
    pub raw_event_clock: u64,
}
#[derive(serde::Deserialize, Clone)]
pub struct InputSignalBagPatch {
    #[serde(rename = "dvce_created_tstamp")]
    timestamp: chrono::DateTime<chrono::Utc>,
    #[serde(rename = "unstruct_event_com_conviva_conviva_video_events_1_0_2.name")]
    pub video_event: Option<String>,
    #[serde(rename = "unstruct_event_com_conviva_raw_event_1_0_1.name")]
    pub raw_event: Option<String>,
}
impl lsp_runtime::WithTimestamp for InputSignalBagPatch {
    fn timestamp(&self) -> lsp_runtime::Timestamp {
        self.timestamp.timestamp_nanos() as u64
    }
}
impl lsp_runtime::InputSignalBag for InputSignalBag {
    type Input = InputSignalBagPatch;
    fn patch(&mut self, patch: InputSignalBagPatch) {
        if let Some(value) = patch.video_event {
            self.video_event_clock += 1;
            self.video_event = value;
        }
        if let Some(value) = patch.raw_event {
            self.raw_event_clock += 1;
            self.raw_event = value;
        }
    }
    fn should_measure(&mut self) -> bool {
        true
    }
}
#[derive(serde::Serialize)]
#[allow(non_snake_case)]
pub struct MetricsBag {
    connectionInducedBufferingTime: u64,
    bufferingTime: u64,
    timeToFirstAttempt: u64,
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
        SignalMapper::new(|lhs: &String| *lhs == "c3.video.attemp")
    };
    let mut __lsp_output_buffer_0;
    let mut __lsp_node_1 = {
        use lsp_component::processors::Latch;
        Latch::<u64>::default()
    };
    let mut __lsp_output_buffer_1;
    let mut __lsp_node_2 = {
        use lsp_component::processors::StateMachine;
        StateMachine::new(0, |_, _| 1)
    };
    let mut __lsp_output_buffer_2;
    let mut __lsp_node_3 = {
        use lsp_component::processors::SignalMapper;
        SignalMapper::new(|lhs: &_| *lhs > 0i32)
    };
    let mut __lsp_output_buffer_3;
    let mut __lsp_node_4 = {
        use lsp_component::measurements::DurationSinceBecomeTrue;
        DurationSinceBecomeTrue::default()
    };
    let mut __lsp_output_buffer_4;
    let mut __lsp_node_5 = {
        use lsp_component::processors::SignalMapper;
        SignalMapper::new(|lhs: &String| *lhs == "c3.video.buffering")
    };
    let mut __lsp_output_buffer_5;
    let mut __lsp_node_6 = {
        use lsp_component::processors::SignalMapper;
        SignalMapper::new(|lhs: &String| *lhs == "c3.video.play")
    };
    let mut __lsp_output_buffer_6;
    let mut __lsp_node_7 = {
        use lsp_component::processors::SignalMapper;
        SignalMapper::new(|(lhs, rhs): &(_, _)| *lhs && *rhs)
    };
    let mut __lsp_output_buffer_7;
    let mut __lsp_node_8 = {
        use lsp_component::processors::Latch;
        Latch::<String>::default()
    };
    let mut __lsp_output_buffer_8;
    let mut __lsp_node_9 = {
        use lsp_component::processors::SignalMapper;
        SignalMapper::new(|lhs: &_| *lhs == "c3.video.play")
    };
    let mut __lsp_output_buffer_9;
    let mut __lsp_node_10 = {
        use lsp_component::measurements::DurationTrue;
        DurationTrue::default()
    };
    let mut __lsp_output_buffer_10;
    let mut __lsp_node_11 = {
        use lsp_component::processors::SignalMapper;
        SignalMapper::new(|lhs: &String| *lhs == "seek_forward")
    };
    let mut __lsp_output_buffer_11;
    let mut __lsp_node_12 = {
        use lsp_component::processors::SignalMapper;
        SignalMapper::new(|lhs: &String| *lhs == "seek_backward")
    };
    let mut __lsp_output_buffer_12;
    let mut __lsp_node_13 = {
        use lsp_component::processors::SignalMapper;
        SignalMapper::new(|(lhs, rhs): &(_, _)| *lhs && *rhs)
    };
    let mut __lsp_output_buffer_13;
    let mut __lsp_node_14 = {
        use lsp_component::processors::SignalMapper;
        SignalMapper::new(|lhs: &_| *lhs == "c3.video.buffering")
    };
    let mut __lsp_output_buffer_14;
    let mut __lsp_node_15 = {
        use lsp_component::processors::SignalMapper;
        SignalMapper::new(|lhs: &_| *lhs == "c3.video.play")
    };
    let mut __lsp_output_buffer_15;
    let mut __lsp_node_16 = {
        use lsp_component::processors::Latch;
        Latch::<bool>::default()
    };
    let mut __lsp_output_buffer_16;
    let mut __lsp_node_17 = {
        use lsp_component::processors::SignalMapper;
        SignalMapper::new(|(lhs, rhs): &(_, _)| *lhs && *rhs)
    };
    let mut __lsp_output_buffer_17;
    let mut __lsp_node_18 = {
        use lsp_component::processors::Latch;
        Latch::with_forget_behavior(
            <bool as Default>::default(),
            <bool as Default>::default(),
            5000000000,
        )
    };
    let mut __lsp_output_buffer_18;
    let mut __lsp_node_19 = {
        use lsp_component::processors::SignalMapper;
        SignalMapper::new(|(lhs, rhs): &(_, _)| *lhs && *rhs)
    };
    let mut __lsp_output_buffer_19;
    let mut __lsp_node_20 = {
        use lsp_component::measurements::DurationSinceBecomeTrue;
        DurationSinceBecomeTrue::default()
    };
    let mut __lsp_output_buffer_20;
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
                    __lsp_node_0.update(&mut update_context, &input_state.video_event);
                instrument_ctx.node_update_end(0usize);
                instrument_ctx.handle_node_output(&__lsp_output_buffer_0);
            }
            {
                use lsp_runtime::measurement::Measurement;
                use lsp_runtime::signal::SignalProcessor;
                instrument_ctx.node_update_begin(1usize);
                __lsp_output_buffer_1 = __lsp_node_1.update(
                    &mut update_context,
                    (&__lsp_output_buffer_0, &input_state.video_event_clock),
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
            }
            {
                use lsp_runtime::measurement::Measurement;
                use lsp_runtime::signal::SignalProcessor;
                instrument_ctx.node_update_begin(4usize);
                __lsp_output_buffer_4 =
                    __lsp_node_4.update(&mut update_context, &__lsp_output_buffer_3);
                instrument_ctx.node_update_end(4usize);
                instrument_ctx.handle_node_output(&__lsp_output_buffer_4);
            }
            {
                use lsp_runtime::measurement::Measurement;
                use lsp_runtime::signal::SignalProcessor;
                instrument_ctx.node_update_begin(5usize);
                __lsp_output_buffer_5 =
                    __lsp_node_5.update(&mut update_context, &input_state.video_event);
                instrument_ctx.node_update_end(5usize);
                instrument_ctx.handle_node_output(&__lsp_output_buffer_5);
            }
            {
                use lsp_runtime::measurement::Measurement;
                use lsp_runtime::signal::SignalProcessor;
                instrument_ctx.node_update_begin(6usize);
                __lsp_output_buffer_6 =
                    __lsp_node_6.update(&mut update_context, &input_state.video_event);
                instrument_ctx.node_update_end(6usize);
                instrument_ctx.handle_node_output(&__lsp_output_buffer_6);
            }
            {
                use lsp_runtime::measurement::Measurement;
                use lsp_runtime::signal::SignalProcessor;
                instrument_ctx.node_update_begin(7usize);
                __lsp_output_buffer_7 = __lsp_node_7.update(
                    &mut update_context,
                    &(__lsp_output_buffer_5, __lsp_output_buffer_6),
                );
                instrument_ctx.node_update_end(7usize);
                instrument_ctx.handle_node_output(&__lsp_output_buffer_7);
            }
            {
                use lsp_runtime::measurement::Measurement;
                use lsp_runtime::signal::SignalProcessor;
                instrument_ctx.node_update_begin(8usize);
                __lsp_output_buffer_8 = __lsp_node_8.update(
                    &mut update_context,
                    (&__lsp_output_buffer_7, &input_state.video_event),
                );
                instrument_ctx.node_update_end(8usize);
                instrument_ctx.handle_node_output(&__lsp_output_buffer_8);
            }
            {
                use lsp_runtime::measurement::Measurement;
                use lsp_runtime::signal::SignalProcessor;
                instrument_ctx.node_update_begin(9usize);
                __lsp_output_buffer_9 =
                    __lsp_node_9.update(&mut update_context, &__lsp_output_buffer_8);
                instrument_ctx.node_update_end(9usize);
                instrument_ctx.handle_node_output(&__lsp_output_buffer_9);
            }
            {
                use lsp_runtime::measurement::Measurement;
                use lsp_runtime::signal::SignalProcessor;
                instrument_ctx.node_update_begin(10usize);
                __lsp_output_buffer_10 =
                    __lsp_node_10.update(&mut update_context, &__lsp_output_buffer_9);
                instrument_ctx.node_update_end(10usize);
                instrument_ctx.handle_node_output(&__lsp_output_buffer_10);
            }
            {
                use lsp_runtime::measurement::Measurement;
                use lsp_runtime::signal::SignalProcessor;
                instrument_ctx.node_update_begin(11usize);
                __lsp_output_buffer_11 =
                    __lsp_node_11.update(&mut update_context, &input_state.raw_event);
                instrument_ctx.node_update_end(11usize);
                instrument_ctx.handle_node_output(&__lsp_output_buffer_11);
            }
            {
                use lsp_runtime::measurement::Measurement;
                use lsp_runtime::signal::SignalProcessor;
                instrument_ctx.node_update_begin(12usize);
                __lsp_output_buffer_12 =
                    __lsp_node_12.update(&mut update_context, &input_state.raw_event);
                instrument_ctx.node_update_end(12usize);
                instrument_ctx.handle_node_output(&__lsp_output_buffer_12);
            }
            {
                use lsp_runtime::measurement::Measurement;
                use lsp_runtime::signal::SignalProcessor;
                instrument_ctx.node_update_begin(13usize);
                __lsp_output_buffer_13 = __lsp_node_13.update(
                    &mut update_context,
                    &(__lsp_output_buffer_11, __lsp_output_buffer_12),
                );
                instrument_ctx.node_update_end(13usize);
                instrument_ctx.handle_node_output(&__lsp_output_buffer_13);
            }
            {
                use lsp_runtime::measurement::Measurement;
                use lsp_runtime::signal::SignalProcessor;
                instrument_ctx.node_update_begin(14usize);
                __lsp_output_buffer_14 =
                    __lsp_node_14.update(&mut update_context, &__lsp_output_buffer_8);
                instrument_ctx.node_update_end(14usize);
                instrument_ctx.handle_node_output(&__lsp_output_buffer_14);
            }
            {
                use lsp_runtime::measurement::Measurement;
                use lsp_runtime::signal::SignalProcessor;
                instrument_ctx.node_update_begin(15usize);
                __lsp_output_buffer_15 =
                    __lsp_node_15.update(&mut update_context, &__lsp_output_buffer_8);
                instrument_ctx.node_update_end(15usize);
                instrument_ctx.handle_node_output(&__lsp_output_buffer_15);
            }
            {
                use lsp_runtime::measurement::Measurement;
                use lsp_runtime::signal::SignalProcessor;
                instrument_ctx.node_update_begin(16usize);
                __lsp_output_buffer_16 = __lsp_node_16.update(
                    &mut update_context,
                    (&__lsp_output_buffer_15, &{
                        let _temp: bool = true;
                        _temp
                    }),
                );
                instrument_ctx.node_update_end(16usize);
                instrument_ctx.handle_node_output(&__lsp_output_buffer_16);
            }
            {
                use lsp_runtime::measurement::Measurement;
                use lsp_runtime::signal::SignalProcessor;
                instrument_ctx.node_update_begin(17usize);
                __lsp_output_buffer_17 = __lsp_node_17.update(
                    &mut update_context,
                    &(__lsp_output_buffer_14, __lsp_output_buffer_16),
                );
                instrument_ctx.node_update_end(17usize);
                instrument_ctx.handle_node_output(&__lsp_output_buffer_17);
            }
            {
                use lsp_runtime::measurement::Measurement;
                use lsp_runtime::signal::SignalProcessor;
                instrument_ctx.node_update_begin(18usize);
                __lsp_output_buffer_18 = __lsp_node_18.update(
                    &mut update_context,
                    (&__lsp_output_buffer_13, &{
                        let _temp: bool = true;
                        _temp
                    }),
                );
                instrument_ctx.node_update_end(18usize);
                instrument_ctx.handle_node_output(&__lsp_output_buffer_18);
            }
            {
                use lsp_runtime::measurement::Measurement;
                use lsp_runtime::signal::SignalProcessor;
                instrument_ctx.node_update_begin(19usize);
                __lsp_output_buffer_19 = __lsp_node_19.update(
                    &mut update_context,
                    &(__lsp_output_buffer_17, __lsp_output_buffer_18),
                );
                instrument_ctx.node_update_end(19usize);
                instrument_ctx.handle_node_output(&__lsp_output_buffer_19);
            }
            {
                use lsp_runtime::measurement::Measurement;
                use lsp_runtime::signal::SignalProcessor;
                instrument_ctx.node_update_begin(20usize);
                __lsp_output_buffer_20 =
                    __lsp_node_20.update(&mut update_context, &__lsp_output_buffer_19);
                instrument_ctx.node_update_end(20usize);
                instrument_ctx.handle_node_output(&__lsp_output_buffer_20);
            };
        }
        if moment.should_take_measurements() {
            let _metrics_bag = MetricsBag {
                connectionInducedBufferingTime: {
                    use lsp_runtime::measurement::Measurement;
                    __lsp_node_20.measure(&mut update_context)
                },
                timeToFirstAttempt: {
                    use lsp_runtime::measurement::Measurement;
                    __lsp_node_4.measure(&mut update_context)
                },
                bufferingTime: {
                    use lsp_runtime::measurement::Measurement;
                    __lsp_node_10.measure(&mut update_context)
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