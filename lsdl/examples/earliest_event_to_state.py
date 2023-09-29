# This is the implementation for https://conviva.atlassian.net/browse/TSA-473

from lsdl.schema import *
from lsdl.modules import *
from lsdl.signal_processors import *
from lsdl import print_ir_to_stdout

class Input(InputSchemaBase):
    event = String()

input = Input()

# This state machine matches pattern "..*" and we check if the state is the state when we seen first state
is_earliest_event = (StateMachine(clock = input.event.clock(), data = Const(1), transition_fn = "|&s:&i32, _| (s+1).min(2)") == 1)
earliest_event_value = Latch(control = is_earliest_event, data = input.event, output_type = "String")
earliest_event_value.add_metric("earliestEventName")

print_ir_to_stdout()
