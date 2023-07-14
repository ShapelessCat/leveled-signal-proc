use std::{fs::File, io::BufReader};
use chrono::{DateTime, Utc};

use lsp::{WithTimestamp, InputState, LspContext, signal::{MappedSignal, ComputedLeveledSignal}, measurement::{DurationTrue, Measurement}};
use serde::Deserialize;
use serde_json::Deserializer;


#[derive(Default, Debug, Clone)]
struct InputType {
    player_state: String,
    network: String,
    cdn: String,
    user_action: String,
}

#[derive(Deserialize, Clone)]
struct EventDataPatch {
    #[serde(rename="newPlayerState")]
    player_state: Option<String>,
    #[serde(rename="newNetwork")]
    network: Option<String>,
    #[serde(rename="newCdn")]
    cdn: Option<String>,
    #[serde(rename="userAction")]
    user_action: Option<String>,
    #[serde(rename = "dateTime")]
    timestamp: DateTime<Utc>,
}

impl WithTimestamp for EventDataPatch {
    fn timestamp(&self) -> u64 {
        self.timestamp.timestamp_nanos() as u64
    }
}

impl InputState for InputType {

    type Event = EventDataPatch;

    fn patch(&mut self, patch: EventDataPatch) {
        patch.player_state.map(|s| self.player_state = s);
        patch.network.map(|s| self.network = s);
        patch.cdn.map(|s| self.cdn = s);
        patch.user_action.map(|s| self.user_action = s);
    }

    fn should_measure(&self) -> bool {
        true
    }
}

fn main() {

    // To simplify the problem, we just assume the data comes from a input file
    let fin = File::open("../input.json").unwrap();
    let reader = BufReader::new(fin);
    let mut ctx = LspContext::<_, InputType>::new(Deserializer::from_reader(reader).into_iter::<EventDataPatch>().filter_map(Result::ok));
    let mut input_state = InputType::default();

    let mut target_signal = MappedSignal::new(|e: &InputType| e.player_state.as_str() == "play" && e.cdn.as_str() == "cdn1" && e.network == "WIFI");
    let mut total_duration = DurationTrue::default();
    let mut target_signal_output = false;
    let mut total_duration_output = 0;

    while let Some(moment) = ctx.next_event(&mut input_state) {
        if moment.should_update_signals() {
            target_signal_output = target_signal.update(ctx.borrow_update_context(), &input_state);
            total_duration.update(ctx.borrow_update_context(), &target_signal_output);
        }

        if moment.should_take_measurements() {
            total_duration_output = total_duration.measure_at(moment.timestamp());
        }
    }
    println!("{}", total_duration_output);
}
