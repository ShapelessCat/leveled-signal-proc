use std::{fs::File, io::BufReader};
use chrono::{DateTime, Utc};

use serde::Deserialize;
use serde_json::Deserializer;
use lsp::{signal::{MappedSignal, LeveledSignal}, measurement::{DurationTrue, Measurement}, Context, WithTimestamp};


// Following code will be generated from the input schema defined in the DSL

#[derive(Deserialize, PartialEq, Eq, Debug, Copy, Clone)]
enum NetworkType {
    Cellular,
    #[serde(rename = "WIFI")]
    Wifi
}
impl Default for NetworkType {
    fn default() -> Self {
        NetworkType::Wifi
    }
}

#[derive(Default, Debug)]
struct EventData {
    timestamp: DateTime<Utc>,
    player_state: String,
    network: NetworkType,
    cdn: String,
    user_action: String,
}

#[derive(Deserialize, Clone)]
struct EventDataPatch {
    #[serde(rename="newPlayerState")]
    player_state: Option<String>,
    #[serde(rename="newNetwork")]
    network: Option<NetworkType>,
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

fn main() {

    // To simplify the problem, we just assume the data comes from a input file
    let fin = File::open("../input.json").unwrap();
    let reader = BufReader::new(fin);
    let mut event_data = EventData::default();

    // The declaration of all the data logic component - will be generated from the DSL
    //let mut target_node = MapSignal::new(|_: u64, input: &EventData| input.player_state.as_str() == "play" && input.cdn.as_str() == "cdn1" && input.network == NetworkType::Wifi);
    //let mut cast_node = MapSignal::new(|_: u64, input| if *input {1.0} else {0.0});
    //let mut integral_node = Integral::default();
    let mut target_signal = MappedSignal::new(|e: &EventData| e.player_state.as_str() == "play" && e.cdn.as_str() == "cdn1" && e.network == NetworkType::Wifi);
    let mut total_duration = DurationTrue::default();
    let mut target_signal_output = false;
    let mut total_duration_output = 0;

    let mut ctx = Context::new(Deserializer::from_reader(reader).into_iter::<EventDataPatch>().filter_map(Result::ok));

    //for patch in Deserializer::from_reader(reader).into_iter::<EventDataPatch>() {
    while let Some(event) = ctx.next_event(){

        let timestamp = event.timestamp();

        if timestamp != event_data.timestamp.timestamp_nanos() as u64 {
            let timestamp = event_data.timestamp.timestamp_nanos() as u64;

            let queue = ctx.update_queue_mut();
            queue.set_epoch(timestamp);

            if event.should_update(){
                target_signal.flush(queue, &mut target_signal_output);
            }
            
            if event.should_measure() {
                total_duration.update(queue, timestamp, &target_signal_output);
                total_duration_output = total_duration.measure_at(timestamp);
            }

            if event.should_update() {
                target_signal.update(queue, &event_data, &mut target_signal_output);
            }
        }

        if let Some(patch) = event.into_input() {
            event_data.player_state = patch.player_state.unwrap_or(event_data.player_state);
            event_data.network = patch.network.unwrap_or(event_data.network);
            event_data.cdn = patch.cdn.unwrap_or(event_data.cdn);
            event_data.user_action = patch.user_action.unwrap_or(event_data.user_action);
            event_data.timestamp = patch.timestamp;
        }
    }
    println!("{}", total_duration_output);
}
