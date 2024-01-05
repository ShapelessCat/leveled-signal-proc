// This example calculates p to e time / user active time
use std::{fs::File, io::BufReader};

use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::Deserializer;

use lsp_component::{
    measurements::{DurationSinceBecomeTrue, PeekTimestamp},
    processors::{
        Accumulator, DurationOfPreviousLevel, Latch, LivenessChecker, SignalMapper, StateMachine,
    },
};
use lsp_component::measurements::combinator::ScopedMeasurement;
use lsp_runtime::{
    InputSignalBag, LspContext, measurement::Measurement, signal::SignalProcessor, Timestamp,
    WithTimestamp,
};

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
        self.timestamp
            .timestamp_nanos_opt()
            .expect("value can not be represented in a timestamp with nanosecond precision.")
            as Timestamp
    }
}

impl InputSignalBag for StateBag {
    type Input = Event;

    fn patch(&mut self, patch: Self::Input) {
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
    let fin = File::open("assets/data/aa-sample.jsonl").unwrap();
    let reader = BufReader::new(fin);
    let mut ctx = LspContext::<_, StateBag>::new(
        Deserializer::from_reader(reader)
            .into_iter::<Event>()
            .filter_map(Result::ok),
        true,
    );

    let mut state = StateBag::default();

    let mut is_heart_beat_mapper = SignalMapper::new(|state: &StateBag| state.user_action != "X");
    let mut is_heart_beat;
    let mut state_watermark_latch = Latch::<Timestamp>::default();
    let mut filtered_hb_signal;
    let mut liveness_signal = LivenessChecker::new(
        |e: &Event| e.user_action.as_ref().map_or(false, |action| action != "X"),
        90_000_000_000,
    );
    let mut is_user_alive = false;

    let mut p_e_state_machine =
        StateMachine::<String, u32, _, Timestamp>::new(0, |&prev_state, input| match prev_state {
            0 if input == "P" => 1,
            1 => {
                if input == "E" {
                    2
                } else {
                    1
                }
            }
            _ => 0,
        });
    let mut p_e_state;

    let mut duration_last_level = DurationOfPreviousLevel::default();
    let mut p_e_state_duration;

    let mut pe_dur_accu = Accumulator::with_event_filter(0, |&s| s == 2);
    let mut pe_duration;

    let mut session_id_acc = Accumulator::with_event_filter(0, |&s| s);
    let mut session_id;

    let mut user_active_time = DurationSinceBecomeTrue::default();

    let mut p_e_seesion = ScopedMeasurement::new(PeekTimestamp);

    let mut first_iter = true;

    while let Some(moment) = ctx.next_event(&mut state) {
        let mut update_ctx = ctx.borrow_update_context();
        if first_iter {
            first_iter = false;
            update_ctx.schedule_measurement(60_000_000_000 - moment.timestamp() % 60_000_000_000);
        }
        if moment.should_update_signals() {
            is_heart_beat = is_heart_beat_mapper.update(&mut update_ctx, &state);
            filtered_hb_signal = state_watermark_latch.update(
                &mut update_ctx,
                (&is_heart_beat, &state.user_action_watermark),
            );
            is_user_alive = liveness_signal.update(&mut update_ctx, &filtered_hb_signal);
            p_e_state = p_e_state_machine.update(
                &mut update_ctx,
                (&state.user_action_watermark, &state.user_action),
            );
            p_e_state_duration = duration_last_level.update(&mut update_ctx, &p_e_state);
            pe_duration = pe_dur_accu.update(&mut update_ctx, (&p_e_state, &p_e_state_duration));
            session_id = session_id_acc.update(&mut update_ctx, (&is_user_alive, &1));
            user_active_time.update(&mut update_ctx, &is_user_alive);
            p_e_seesion.update(&mut update_ctx, (&session_id, &pe_duration));
        }
        if moment.should_take_measurements() {
            let user_active_time = user_active_time.measure(&mut update_ctx);
            let pe_time = p_e_seesion.measure(&mut update_ctx);

            if moment.timestamp() % 60_000_000_000 == 0 {
                println!("1min summary");
                update_ctx.schedule_measurement(60_000_000_000);
            }

            println!(
                "{} {}",
                moment.timestamp(),
                if user_active_time > 0 {
                    pe_time as f64 / user_active_time as f64
                } else {
                    0.0
                }
            );
        }
    }
}
