from lsdl import print_ir_to_stdout
from lsdl.lsp_model import named, InputSchemaBase, String


class InputSignal(InputSchemaBase):
    _timestamp_key = "dateTime"

    player_state = named("newPlayerState", String())  # noqa: E221
    network      = named("newNetwork",     String())  # noqa: E221
    cdn          = named("newCdn",         String())  # noqa: E221
    user_action  = named("newUserAction",  String())  # noqa: E221


input_signal = InputSignal()

target = (
    (input_signal.user_action == "play").has_been_true() &
    ~((input_signal.user_action == "seek").has_been_true(5_000_000_000)) &
    (input_signal.player_state == "buffer") &
    (input_signal.cdn == "cdn1")
)
target.measure_duration_true().add_metric("totalPlayTime")

print_ir_to_stdout()
