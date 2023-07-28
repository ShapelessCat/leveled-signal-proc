/// This impelements the DAG that is used in TLB2 profiling (See https://conviva.atlassian.net/wiki/spaces/~589178245/pages/1867646607/DAG-level+instrumentation for details)
/// Please note that this file will be automatically generate from the LSP DSL in the formal LSP system. 
/// Currently this file is hand written for demostration purposes.
/// 
///     dag:
///       rawEvents:
///         op: eventSourceTimeline
///         source: $videoHeartbeats
///       userAction:
///         op: latestEventToState
///         in:
///           op: get
///           in: $rawEvents
///           path: "userAction"
///       playerState:
///         op: latestEventToState
///         in:
///           op: get
///           in: $rawEvents
///           path: "newPlayerState"
///       network:
///         op: latestEventToState
///         in:
///           op: get
///           in: $rawEvents
///           path: "newNetwork"
///       cdn:
///         op: latestEventToState
///         in:
///           op: get
///           in: $rawEvents
///           path: "newCdn"
///       isPlay:
///         op: equals
///         left: $playerState
///         right: "play"
///       isWifi:
///         op: equals
///         left: $network
///         right: "WIFI"
///       isCDN1:
///         op: equals
///         left: $cdn
///         right: "cdn1"
///       target:
///         op: and
///         args:
///           - $isPlay
///           - $isWifi
///           - $isCDN1
///       totalTime:///       evaluatedInRealtime:
///         op: evaluateAt
///         in: $totalTime
///         evaluationPoints: $rawEvents
///         op: durationTrue
///         in: $target
///         slidingWindow: +inf
///       evaluatedInRealtime:
///         op: evaluateAt
///         in: $totalTime
///         evaluationPoints: $rawEvents

use std::{fs::File, io::{BufReader, BufWriter, Write}};
use chrono::{DateTime, Utc};

use lsp_component::{processors::SignalMapper, measurements::DurationTrue};
use lsp_runtime::{WithTimestamp, InputState, LspContext, measurement::Measurement, signal::SingnalProcessor};
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

    fn should_measure(&mut self) -> bool {
        true
    }
}

#[allow(unused_assignments)]
fn main() {

    // To simplify the problem, we just assume the data comes from a input file
    let fin = File::open("../input.json").unwrap();
    let mut fout = BufWriter::new(File::open("/dev/null").unwrap());
    let reader = BufReader::new(fin);
    if std::env::var("PARSING_ONLY").is_ok()  {
        println!("{}", Deserializer::from_reader(reader).into_iter::<EventDataPatch>().filter_map(Result::ok).count());
        return;
    }
    let mut ctx = LspContext::<_, InputType>::new(Deserializer::from_reader(reader).into_iter::<EventDataPatch>().filter_map(Result::ok));
    let mut input_state = InputType::default();

    let mut target_signal = SignalMapper::new(|e: &InputType| e.player_state.as_str() == "play" && e.cdn.as_str() == "cdn1" && e.network == "WIFI");
    let mut total_duration = DurationTrue::default();
    let mut target_signal_output = false;
    let mut total_duration_output = 0;

    let mut time_ops = 0.0;

    while let Some(moment) = ctx.next_event(&mut input_state) {

        let start_ts = std::time::Instant::now();

        let mut uc = ctx.borrow_update_context();
        if moment.should_update_signals() {
            target_signal_output = target_signal.update(&mut uc, &input_state);
            total_duration.update(&mut uc, &target_signal_output);
        }

        if moment.should_take_measurements() {
            total_duration_output = total_duration.measure(&mut uc);
            write!(fout, "{}", total_duration_output).ok();
        }
        
        let end_ts = std::time::Instant::now();
        time_ops += end_ts.duration_since(start_ts).as_secs_f64();
    }

    println!("{}", time_ops);

    //println!("{}", total_duration_output);
}
