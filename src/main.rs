use time_series_proc::{SignalProcessor, MapSignal, Integral};

use std::{fs::File, io::BufReader};
use chrono::{DateTime, Utc};

use serde::Deserialize;
use serde_json::Deserializer;

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

fn main() {
    let fin = File::open("../input.json").unwrap();
    let reader = BufReader::new(fin);
    let mut event_data = EventData::default();

    let mut check_node = MapSignal::new(|_: u64, input: &EventData| input.player_state.as_str() == "play" && input.cdn.as_str() == "cdn1" && input.network == NetworkType::Wifi);
    let mut cast_node = MapSignal::new(|_: u64, input: &bool| if *input {1.0} else {0.0});
    let mut integral_node = Integral::default();

    let mut res = 0.0;

    for patch in Deserializer::from_reader(reader).into_iter::<EventDataPatch>() {
        let patch = patch.unwrap();

        if patch.timestamp != event_data.timestamp {

            let timestamp = event_data.timestamp.timestamp_nanos() as u64;
            let v0 = check_node.apply(timestamp, &event_data).unwrap().unwrap();
            let v1 = cast_node.apply(timestamp, &v0).unwrap().unwrap();
            let v2 = integral_node.apply(timestamp, &v1).unwrap().unwrap();
            res = v2;
        }
        
        event_data.player_state = patch.player_state.unwrap_or(event_data.player_state);
        event_data.network = patch.network.unwrap_or(event_data.network);
        event_data.cdn = patch.cdn.unwrap_or(event_data.cdn);
        event_data.user_action = patch.user_action.unwrap_or(event_data.user_action);
        event_data.timestamp = patch.timestamp;
    }
    println!("{}", res);
}
