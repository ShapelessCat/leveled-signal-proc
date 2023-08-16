from lsdl.schema import *
from lsdl.modules import *
from lsdl import print_ir_to_stdout

class Input(InputSchemaBase):
    user_action = named("userAction", String())
    page        = String()

input = Input()

event_filter(
    event_signal = input.user_action,
    event_value = "P",
).count_changes().add_metric("bufferCount")

print_ir_to_stdout()