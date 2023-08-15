// This is an example which matches event pattern as A(BC)*D and report
// the total duration between first B after A and D. For example
// A B C B C X B X C D
//   |---------------|
//     duration between this
// But this case don't count
// A B C B C B D
// because it doesn't match the pattern A(BC)*D
//
// The proposed DSL to generate this Rust code:
//
// class StateBag(schema.InputStateBase):
//     user_action = schema.String(id = "userAction")
// input = StateBag()
// event_of_interest = event_filter(input = input.user_action, values = ["A", "B", "C", "D"])
// state_machine = state_machine(input = event_of_interest, pattern_regex = "A(B|C)*D")
// b_to_d_duration = total_duration_of_state(input = state_machine, min_state = 1, max_state = 4)
// measurements.peek_value(var = b_to_d_duration, key = "b_to_d_duration")

use std::{fs::File, io::BufReader};

use chrono::{DateTime, Utc};
use lsp_component::{
    measurements::Peek,
    processors::{Accumulator, DurationOfPreviousLevel, Latch, SignalMapper, StateMachine},
};
use lsp_runtime::{
    measurement::Measurement, signal::SignalProcessor, InputSignalBag, LspContext, Timestamp,
    WithTimestamp,
};
use serde::Deserialize;
use serde_json::Deserializer;

#[derive(Default, Clone, Debug)]
struct StateBag {
    user_action: String,
    user_action_watermark: Timestamp,
}

#[derive(Deserialize)]
struct Event {
    timestamp: DateTime<Utc>,
    #[serde(rename = "userAction")]
    user_action: Option<String>,
}

impl WithTimestamp for Event {
    fn timestamp(&self) -> Timestamp {
        self.timestamp.timestamp_nanos() as Timestamp
    }
}

impl InputSignalBag for StateBag {
    type Input = Event;

    fn patch(&mut self, patch: Self::Input) {
        let ts = patch.timestamp();
        if let Some(user_action) = patch.user_action {
            self.user_action = user_action;
            self.user_action_watermark = ts;
        }
    }

    fn should_measure(&mut self) -> bool {
        true
    }
}

// This is a state machine that matches A(BC)*D
// Actually there are algorithm which can automatically generate DFA from a regular expression
// Thus in the code generator, the API would be like
// macth_event_seq("A(BC)*D")
fn state_transit(state: u32, input: &String) -> u32 {
    let next_state = match state {
        0 => {
            if input == "A" {
                1
            } else {
                0
            }
        }
        1 => {
            if input == "B" {
                2
            } else {
                0
            }
        }
        2 => {
            if input == "C" {
                3
            } else {
                0
            }
        }
        3 => {
            if input == "D" {
                4
            } else if input == "B" {
                2
            } else {
                0
            }
        }
        4 => 0,
        _ => unreachable!(),
    };
    if next_state == 0 && input == "A" {
        1
    } else {
        next_state
    }
}

fn main() {
    let fin = File::open("data/state-machine.json").unwrap();
    let reader = BufReader::new(fin);
    let mut ctx = LspContext::<_, StateBag>::new(
        Deserializer::from_reader(reader)
            .into_iter::<Event>()
            .filter_map(Result::ok),
    );

    let mut state = StateBag::default();

    let mut event_filter = SignalMapper::new(|input: &StateBag| {
        matches!(input.user_action.as_str(), "A" | "B" | "C" | "D")
    });
    let mut event_filter_output;

    let mut event_filter_latch = Latch::<Timestamp>::default();
    let mut event_filter_latch_output;

    let mut state_machine = StateMachine::new(0, |s, i| state_transit(*s, i));
    let mut state_machine_output;

    let mut is_b_to_d_mapper = SignalMapper::new(|&s| s > 0 && s < 4);
    let mut is_b_to_d;

    let mut duration_b_to_d_last_level = DurationOfPreviousLevel::default();
    let mut duration_b_to_d;

    let mut b_to_d_acc = Accumulator::with_event_filter(0, |&s| s == 4);
    let mut b_to_d_duration_acc;

    let mut peek_b_to_d = Peek::default();

    while let Some(moment) = ctx.next_event(&mut state) {
        let mut uc = ctx.borrow_update_context();
        if moment.should_update_signals() {
            event_filter_output = event_filter.update(&mut uc, &state);
            event_filter_latch_output = event_filter_latch.update(
                &mut uc,
                (&event_filter_output, &state.user_action_watermark),
            );
            state_machine_output =
                state_machine.update(&mut uc, (&event_filter_latch_output, &state.user_action));
            is_b_to_d = is_b_to_d_mapper.update(&mut uc, &state_machine_output);
            duration_b_to_d = duration_b_to_d_last_level.update(&mut uc, &is_b_to_d);
            b_to_d_duration_acc =
                b_to_d_acc.update(&mut uc, (&state_machine_output, &duration_b_to_d));
            peek_b_to_d.update(&mut uc, &b_to_d_duration_acc);
        }

        if moment.should_take_measurements() {
            let b_to_d_sum = peek_b_to_d.measure(&mut uc);
            println!("{} {:?}", b_to_d_sum, state);
        }
    }
}
