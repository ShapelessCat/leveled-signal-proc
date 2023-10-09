from lsdl.prelude import *


class InputSignal(InputSchemaBase):
    _timestamp_key = "dateTime"
    player_state = named("newPlayerState", String())
    network      = named("newNetwork",     String())
    cdn          = named("newCdn",         String())
    user_action  = named("newUserAction",  String())

input_signal = InputSignal()

((input_signal.player_state == "play") &
 (input_signal.cdn == "cdn1") &
 (input_signal.network == "WIFI"))\
    .measure_duration_true() \
    .add_metric("playtime")

print_ir_to_stdout()
