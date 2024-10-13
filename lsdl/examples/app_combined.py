from lsdl import print_ir_to_stdout
from lsdl.lsp_model import InputSchemaBase, named
from lsdl.processors import Const, SignalFilterBuilder, StateMachine


# First, we need define the input schema
class InputSignal(InputSchemaBase):
    # We can override the key for the timestamp
    _timestamp_key = "dvce_created_tstamp"

    # Define the member of the schemas:
    # Function `named` maps the key in the input data to the member name.
    video_event = named(
        "unstruct_event_com_conviva_conviva_video_events_1_0_2.name"
    )  # noqa: E221, E501
    raw_event = named(
        "unstruct_event_com_conviva_raw_event_1_0_1.name"
    )  # noqa: E221, E501


# Then, let's instantiate the input schema
input_signal = InputSignal()

# We can also define constants for code conciseness
VE_ATTEMPT = "c3.video.attempt"
VE_BUFFER = "c3.video.buffering"
VE_PLAY = "c3.video.play"

RE_SEEK_F = "seek_forward"
RE_SEEK_B = "seek_backward"

# At this point we filter the input signal to only include the attempt events.
attempt_event = (
    SignalFilterBuilder(input_signal.video_event)
    .filter_values(VE_ATTEMPT)
    .build_clock_filter()
)

# Then we use a state machine to match the event pattern ".+", which indicates
# that we have seen at least 1 attempt event
has_attempted = (
    StateMachine(clock=attempt_event, data=Const(1), transition_fn="|_, _| 1") > 0
)
# Finally we measure the duration since the has_attempt signal becomes true
has_attempted.measure_duration_since_true().add_metric("timeToFirstAttempt")

# At this point we filter the input signal to only include the play or
# buffering events. Note that we call `.build_value_filter`, which outputs a
# signal of value rather than clock. This is more like TLB2's state concept,
# the player_state is a signal that tracking current player state.
player_state = (
    SignalFilterBuilder(input_signal.video_event)
    .filter_values(VE_BUFFER, VE_PLAY)
    .build_value_filter()
)

# So the buffering time is simply defined as the duration of the player_state
# being "buffering"
(player_state == VE_BUFFER).measure_duration_true().add_metric("bufferingTime")

# we can define the connection induced buffering as following:
#   1. The player_state is "buffering"
#   2. The player_state has been "play" before
#   3. The user is seeking within 5s
seeking = (input_signal.raw_event == RE_SEEK_F) | (input_signal.raw_event == RE_SEEK_B)
is_buffering = player_state == VE_BUFFER
is_playing = player_state == VE_PLAY
(
    is_buffering & is_playing.has_been_true() & seeking.has_been_true(duration="5s")
).measure_duration_since_true().add_metric("connectionInducedBufferingTime")

print_ir_to_stdout()
