from lsdl import print_ir_to_stdout
from lsdl.lsp_model import named, InputSchemaBase, String
from lsdl.processors import SignalFilterBuilder


class InputSignal(InputSchemaBase):
    user_action = named("userAction", String())  # noqa: E221
    page        = String()                             # noqa: E221


input_signal = InputSignal()

SignalFilterBuilder(input_signal.user_action)\
    .filter_values("P")\
    .build_clock_filter()\
    .count_changes()\
    .add_metric("pCount")

print_ir_to_stdout()
