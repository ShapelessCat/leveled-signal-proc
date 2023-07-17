/// The LSPDSL to generate the code might look like following
/// 
/// class StateBag(InputStateBase):
///     page = schema.String()
///     user_action = schema.String()
/// 
/// input = StateBag()
/// liveness_signal = liveness_analysis(input.user_action, RustEnv("action").code('action != 'x''))
/// count_c = counter(event_filter(input.user_action, RustEnv("what").code('what == "c"')))
/// count_all = counter(input.user_action.watermark)
/// 
/// measure_count_alive(count_c, liveness_signal)

use std::{fs::File, io::BufReader};

use chrono::{DateTime, Utc};
use lsp_component::processors::{SignalMapper, LivenessChecker, Latch, DurationOfPreviousLevel, ValueChangeCounter, StateMachine, Accumulator};
use lsp_runtime::{WithTimestamp, Timestamp, InputState, LspContext, signal::SingnalProcessor};
use serde::Deserialize;
use serde_json::Deserializer;

#[derive(Default, Clone, Debug)]
struct StateBag {
    pending_measure: bool,
    page: String,
    page_watermark: Timestamp,
    user_action: String,
    user_action_watermark: Timestamp,
}

#[derive(Deserialize)]
struct Event {
    timestamp: DateTime<Utc>,
    page: Option<String>,
    #[serde(rename = "userAction")]
    user_action: Option<String>,
}

impl WithTimestamp for Event {
    fn timestamp(&self) -> Timestamp {
        self.timestamp.timestamp_nanos() as u64
    }
}

impl InputState for StateBag {
    type Event = Event;

    fn patch(&mut self, patch: Self::Event) {
        let ts = patch.timestamp();
        if let Some(page) = patch.page {
            self.pending_measure = true;
            self.page_watermark = ts;
            self.page = page;
        }
        if let Some(user_aciton) = patch.user_action {
            self.user_action_watermark = ts;
            self.user_action = user_aciton;
        }
    }
    fn should_measure(&mut self) -> bool {
        if self.pending_measure {
            self.pending_measure = false;
            return true;
        }
        false
    }
}

#[allow(unused_assignments)]
fn main() {
    let fin = File::open("data/aa-sample.json").unwrap();
    let reader = BufReader::new(fin);
    let mut ctx = LspContext::<_, StateBag>::new(Deserializer::from_reader(reader).into_iter::<Event>().filter_map(Result::ok));

    let mut state = StateBag::default();

    let mut is_heart_beat_mapper = SignalMapper::new(|state: &StateBag| state.user_action != "X");
    let mut state_watermark_latch = Latch::<Timestamp>::default();
    let mut liveness_signal = LivenessChecker::new(|e: &Event| e.user_action.as_ref().map_or(false, |action| action != "X"), 90_000_000_000);

    let mut is_c_mapper = SignalMapper::new(|state: &StateBag| state.user_action == "C");
    let mut c_filter_latch = Latch::<Timestamp>::default();
    let mut c_counter = ValueChangeCounter::with_init_value(0);

    let mut all_counter = ValueChangeCounter::with_init_value(0);

    let mut p_e_state_machine = StateMachine::<String, u32, _, Timestamp>::new(0, |&prev_state, input| {
        if prev_state == 0 {
            if input.as_str() == "P" {
                1
            } else {
                0
            }
        } else if prev_state == 1 {
            if input.as_str() == "E" {
                2
            } else {
                1
            }
        } else {
            0
        }
    });

    let mut p_e_state_filter = SignalMapper::new(|&s| s == 2);
    let mut p_e_event_latch = Latch::<Timestamp>::default();

    let mut p_e_duration_accmulator = Accumulator::new(0);

    let mut p_e_level_duration = DurationOfPreviousLevel::default();

    let mut is_heart_beat_output = false;
    let mut state_watermark_latch_output = 0;
    let mut liveness_signal_output = false;
    let mut is_c_mapper_output = false;
    let mut c_filter_latch_output = 0;
    let mut c_counter_output = 0;
    let mut all_counter_output = 0;

    let mut p_e_state_machine_output = 0;
    let mut p_e_level_duration_output = 0;
    let mut p_e_duration_accu_output = 0;
    let mut p_e_state_filter_output = false;
    let mut p_e_event_latch_output = 0;

    while let Some(moment) = ctx.next_event(&mut state) {
        if moment.should_update_signals() {
            let mut update_ctx = ctx.borrow_update_context();
            is_heart_beat_output = is_heart_beat_mapper.update(&mut update_ctx, &state);
            state_watermark_latch_output = state_watermark_latch.update(&mut update_ctx, &(is_heart_beat_output, state.user_action_watermark));
            liveness_signal_output = liveness_signal.update(&mut update_ctx, &state_watermark_latch_output);

            is_c_mapper_output = is_c_mapper.update(&mut update_ctx, &state);
            c_filter_latch_output = c_filter_latch.update(&mut update_ctx, &(is_c_mapper_output, state.user_action_watermark));
            c_counter_output = c_counter.update(&mut update_ctx, &c_filter_latch_output);

            all_counter_output = all_counter.update(&mut update_ctx, &state.user_action_watermark);

            p_e_state_machine_output = p_e_state_machine.update(&mut update_ctx, &(state.user_action_watermark, state.user_action.clone()));

            p_e_level_duration_output = p_e_level_duration.update(&mut update_ctx, &p_e_state_machine_output);
            p_e_state_filter_output = p_e_state_filter.update(&mut update_ctx, &p_e_state_machine_output);
            p_e_event_latch_output = p_e_event_latch.update(&mut update_ctx, &(p_e_state_filter_output, state.user_action_watermark));
            p_e_duration_accu_output = p_e_duration_accmulator.update(&mut update_ctx, &(p_e_event_latch_output, p_e_level_duration_output));
        }
        println!("{} {} {} {} {} {} {} {} {:?}", 
            moment.should_take_measurements(), 
            liveness_signal_output, 
            c_counter_output, 
            c_filter_latch_output, 
            all_counter_output, 
            p_e_state_machine_output, 
            p_e_level_duration_output, 
            p_e_duration_accu_output,
            state);
    }

}