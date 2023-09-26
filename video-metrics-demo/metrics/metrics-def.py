from lsdl.schema import InputSchemaBase, named, String, Integer
from lsdl import print_ir_to_stdout
from lsdl.signal_processors import StateMachine

PS_PLAYING = "playing"
PS_BUFFERING = "buffering"
PS_PAUSE = "pause"

EV_SEEK_S = "seek start"
EV_SEEK_E = "seek end"

class Input(InputSchemaBase):
    _timestamp_key = "timestamp"
    session_id     = named("sessionId",   String())
    player_state   = named("PlayerState", String())
    cdn            = named("CDN",         String())
    bit_rate       = named("BitRate",     Integer())
    ev             = named("ev",          String())

input = Input()

num_sid = input.session_id.count_changes()
is_buffering = (input.player_state == PS_BUFFERING)

num_ps = input.player_state.map(bind_var = "s", lambda_src = f"""
    match s.as_str() {{
        "{PS_PLAYING}" => 0,
        "{PS_BUFFERING}" => 1,
        _ => 2,
    }}
""")

init_play_state = StateMachine(input.session_id.clock(), [num_sid, num_ps], transition_fn = """
    |state, &(sid, ps)| {
        let last_sid = state >> 3;
        let last_state = if sid != last_sid { 0 } else { state & 0x7 };
        let new_state = match last_state {
            0 => if ps != 0 { 0 } else { 1 },
            _ => 1,       
        };
        (sid << 3) + new_state
    }
""").map(bind_var="value" , lambda_src="value & 0x7")
                               
is_init_buffering = ((init_play_state == 0) & is_buffering)

is_init_buffering.measure_duration_true(scope_signal = num_sid).add_metric("initBufferingTime")
is_buffering.measure_duration_true(scope_signal = num_sid).add_metric("bufferingTime")

print_ir_to_stdout()