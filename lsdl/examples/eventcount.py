from lsdl.prelude import *


class InputSignal(InputSchemaBase):
    user_action = named("userAction", String())
    page        = String()

input_signal = InputSignal()

SignalFilterBuilder(input_signal.user_action)\
    .filter_values("P")\
    .build_clock_filter()\
    .count_changes()\
    .add_metric("pCount")

print_ir_to_stdout()
