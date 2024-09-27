from lsdl import print_ir_to_stdout
from lsdl.lsp_model import named, InputSchemaBase


class InputSignal(InputSchemaBase):
    _timestamp_key = "dateTime"

    player_state = named("newPlayerState")  # noqa: E221
    network = named("newNetwork")  # noqa: E221
    cdn = named("newCdn")  # noqa: E221
    user_action = named("newUserAction")  # noqa: E221


input_signal = InputSignal()

(
    (input_signal.player_state == "play")
    & (input_signal.cdn == "cdn1")
    & (input_signal.network == "WIFI")
).measure_duration_true().add_metric("playtime")

print_ir_to_stdout()
