from lsdl.const import Const
from lsdl.signal_processors import StateMachine
from lsdl.schema import *
from lsdl.modules import *
from lsdl import print_ir_to_stdout

# First, we need define the input schema
class Input(InputSchemaBase):
    # We can override the key for the timestamp
    _timestamp_key = "dvce_created_tstamp"
    # Then we can define the member of the schemas. 
    # Function named is used to map the key in the input data to the member name.
    video_event = named("unstruct_event_com_conviva_conviva_video_events_1_0_2.name", String())
    raw_event   = named("unstruct_event_com_conviva_raw_event_1_0_1.name", String())

# Then, let's instantiate the input schema
input = Input()

# We can also define constants for code conciseness
VE_ATEMPT = "c3.video.attemp"
VE_BUFFER = "c3.video.buffering"
VE_PLAY   = "c3.video.play"
RE_SEEK_F = "seek_forward"
RE_SEEK_B = "seek_backward"

# At this point we filter the input signal to only include the attempt events.
attempt_event = SignalFilterBuilder(input.video_event).filter_values(VE_ATEMPT).build_clock_filter()
# Then we use a state machine to match the event pattern ".+", which indicates that we have seen at least 1 attempt event 
has_attempted = StateMachine(clock = attempt_event, data = Const(1), transition_fn = "|_, _| 1") > 0
# Finally we measure the duration since the has_attempt signal becomes true
has_attempted.measure_duration_since_true().add_metric("timeToFirstAttempt")

# At this point we filter the input signal to only include the play or buffering events.
# Note that the filter is set to value mode, which outputs a value rather than a clock signal.
# This is more like TLB2's state concept, the player_state is a signal that tracking current player state. 
player_state = SignalFilterBuilder(input.video_event).filter_values(VE_BUFFER, VE_PLAY).build_value_filter()

# So the buffering time is simply defined as the duration of the player_state being "buffering"
(player_state == VE_PLAY).measure_duration_true().add_metric("bufferingTime")

# we can define the connection induced buffering as following:
#   1. The player_state is "buffering"
#   2. The player_state has been "play" before
#   3. The user is seeking within 5s
seeking = (input.raw_event == RE_SEEK_F) | (input.raw_event == RE_SEEK_B)
is_buffering = player_state == VE_BUFFER
(is_buffering & has_been_true(player_state == VE_PLAY) & has_been_true(seeking, duration = "5s"))\
    .measure_duration_since_true().add_metric("connectionInducedBufferingTime")

print_ir_to_stdout()