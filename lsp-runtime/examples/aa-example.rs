use std::{fs::File, io::BufReader};

use chrono::{DateTime, Utc};
use lsp_runtime::{WithTimestamp, Timestamp, InputState, LspContext, signal::{MappedSignal, Latch, ComputedLeveledSignal, LivenessSignal, ValueChangeCounter}};
use serde::Deserialize;
use serde_json::Deserializer;

#[derive(Default, Clone, Debug)]
struct StateBag {
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
            self.page_watermark = ts;
            self.page = page;
        }
        if let Some(user_aciton) = patch.user_action {
            self.user_action_watermark = ts;
            self.user_action = user_aciton;
        }
    }
}

#[allow(unused_assignments)]
fn main() {
    let fin = File::open("data/aa-sample.json").unwrap();
    let reader = BufReader::new(fin);
    let mut ctx = LspContext::<_, StateBag>::new(Deserializer::from_reader(reader).into_iter::<Event>().filter_map(Result::ok));

    let mut state = StateBag::default();

    let mut is_heart_beat_mapper = MappedSignal::new(|state: &StateBag| state.user_action != "X");
    let mut state_watermark_latch = Latch::<Timestamp>::default();
    let mut liveness_signal = LivenessSignal::new(|e: &Event| e.user_action.as_ref().map_or(false, |action| action != "X"), 90_000_000_000);

    let mut is_c_mapper = MappedSignal::new(|state: &StateBag| state.user_action == "C");
    let mut c_filter_latch = Latch::<Timestamp>::default();
    let mut c_counter = ValueChangeCounter::with_init_value(0);

    let mut all_counter = ValueChangeCounter::with_init_value(0);

    let mut is_heart_beat_output = false;
    let mut state_watermark_latch_output = 0;
    let mut liveness_signal_output = false;
    let mut is_c_mapper_output = false;
    let mut c_filter_latch_output = 0;
    let mut c_counter_output = 0;
    let mut all_counter_output = 0;

    while let Some(moment) = ctx.next_event(&mut state) {
        if moment.should_update_signals() {
            is_heart_beat_output = is_heart_beat_mapper.update(ctx.borrow_update_context(), &state);
            state_watermark_latch_output = state_watermark_latch.update(ctx.borrow_update_context(), &(is_heart_beat_output, state.user_action_watermark));
            liveness_signal_output = liveness_signal.update(ctx.borrow_update_context(), &state_watermark_latch_output);

            is_c_mapper_output = is_c_mapper.update(ctx.borrow_update_context(), &state);
            c_filter_latch_output = c_filter_latch.update(ctx.borrow_update_context(), &(is_c_mapper_output, state.user_action_watermark));
            c_counter_output = c_counter.update(ctx.borrow_update_context(), &c_filter_latch_output);

            all_counter_output = all_counter.update(ctx.borrow_update_context(), &state.user_action_watermark);
        }
        println!("{} {} {} {} {} {:?}", moment.timestamp(), liveness_signal_output, c_counter_output, c_filter_latch_output, all_counter_output, state);
    }

}
