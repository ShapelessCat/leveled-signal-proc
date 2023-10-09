from lsdl.prelude import *


class InputSignal(InputSchemaBase):
    _timestamp_key = "dateTime"
    player_state = named("newPlayerState", String())
    network      = named("newNetwork",     String())
    cdn          = named("newCdn",         String())
    user_action  = named("newUserAction",  String())

input_signal = InputSignal()

target = has_been_true(input_signal.user_action == "play") &\
      ~has_been_true(input_signal.user_action == "seek", 5_000_000_000) &\
        (input_signal.player_state == "buffer") &\
        (input_signal.cdn == "cdn1")
target.measure_duration_true().add_metric("totalPlayTime")

print_ir_to_stdout()
