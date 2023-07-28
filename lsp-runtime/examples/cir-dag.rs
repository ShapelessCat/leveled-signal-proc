use std::{fs::File, io::{BufReader, BufWriter, Write}};
use chrono::{DateTime, Utc};

use lsp_component::{processors::{SignalMapper, Latch}, measurements::DurationTrue};
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

    let mut has_started_mapper = SignalMapper::new(|input: &InputType| input.user_action == "play");
    let mut has_started_mapper_output;

    let mut has_started_latch = Latch::<bool>::default();
    let mut has_started;

    let mut has_seeked_mapper = SignalMapper::new(|input: &InputType| input.user_action == "seek");
    let mut has_seeked_mapper_output;

    let mut has_seeked_latch = Latch::with_forget_behavior(false, false, 5_000_000_000);
    let mut has_seeked;

    let mut is_buffered_mapper = SignalMapper::new(|input: &InputType| input.player_state == "buffer");
    let mut is_buffer;

    let mut is_cdn1_mapper = SignalMapper::new(|input: &InputType| input.cdn == "cdn1");
    let mut is_cdn1;

    let mut target_mapper = SignalMapper::new(|&(has_started, has_seeked, is_buffer, is_cdn1): &(bool, bool, bool, bool)| is_buffer && has_started && is_cdn1 && !has_seeked);

    let mut total_duration = DurationTrue::default();
    let mut target_signal_output = false;
    let mut total_duration_output = 0;

    let mut time_ops = 0.0;

    while let Some(moment) = ctx.next_event(&mut input_state) {

        let start_ts = std::time::Instant::now();

        let mut uc = ctx.borrow_update_context();
        if moment.should_update_signals() {
            has_started_mapper_output = has_started_mapper.update(&mut uc, &input_state);
            has_started = has_started_latch.update(&mut uc, (&has_started_mapper_output, &true));
            has_seeked_mapper_output = has_seeked_mapper.update(&mut uc, &input_state);
            has_seeked = has_seeked_latch.update(&mut uc, (&has_seeked_mapper_output, &true));
            is_buffer = is_buffered_mapper.update(&mut uc, &input_state);
            is_cdn1 = is_cdn1_mapper.update(&mut uc, &input_state);
            target_signal_output = target_mapper.update(&mut uc, &(has_started, has_seeked, is_buffer, is_cdn1));
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

}