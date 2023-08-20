from lsdl.schema import *
from lsdl.modules import *
from lsdl import print_ir_to_stdout

class Input(InputSchemaBase):
    user_action = named("userAction", String())
    page        = String()

input = Input()

SignalFilterBuilder(input.user_action)\
    .filter_values("P")\
    .build_clock_filter()\
    .count_changes()\
    .add_metric("pCount")

print_ir_to_stdout()
