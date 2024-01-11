from lsdl.prelude import named, print_ir_to_stdout, InputSchemaBase, String


class InputSignal(InputSchemaBase):
    _timestamp_key = "dateTime"

    player_state = named("newPlayerState", String())  # noqa: E221
    network      = named("newNetwork",     String())  # noqa: E221
    cdn          = named("newCdn",         String())  # noqa: E221
    user_action  = named("newUserAction",  String())  # noqa: E221


input_signal = InputSignal()

((input_signal.player_state == "play") &
 (input_signal.cdn == "cdn1") &
 (input_signal.network == "WIFI")) \
    .measure_duration_true() \
    .add_metric("playtime")

print_ir_to_stdout()
