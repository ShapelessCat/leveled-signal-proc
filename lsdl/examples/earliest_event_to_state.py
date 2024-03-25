"""Implementation for https://conviva.atlassian.net/browse/TSA-473"""

from lsdl import print_ir_to_stdout
from lsdl.lsp_model import InputSchemaBase, String
from lsdl.processors import Const, LevelTriggeredLatch, StateMachine


class InputSignal(InputSchemaBase):
    event = String()


input_signal = InputSignal()

# This state machine matches pattern "..*", and we check if the state is the
# state when we see first state
is_earliest_event = StateMachine(
    clock=input_signal.event.clock(),
    data=Const(1),
    transition_fn="|&s:&i32, _| (s+1).min(2)"
) == 1
earliest_event_value = LevelTriggeredLatch(control=is_earliest_event,
                             data=input_signal.event)
earliest_event_value.add_metric("earliestEventName")

print_ir_to_stdout()
