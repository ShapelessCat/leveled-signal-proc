from lsdl.schema import InputSchemaBase, named, String, Integer
from lsdl.signal_processors import LivenessChecker, EdgeTriggeredLatch, SignalMapper
from lsdl import print_ir_to_stdout
from lsdl.const import Const
from lsdl.modules import make_tuple

class Input(InputSchemaBase):
    _timestamp_key = "timestamp"
    player_state   = named("PlayerState", String())
    cdn            = named("CDN",         String())
    bit_rate       = named("BitRate",     Integer())
    page_id        = named("pageId",      String())
    ev             = String()
    special_event  = String()

input = Input()

is_session_alive = EdgeTriggeredLatch(input.special_event.clock(), data = Const(True), forget_duration = 90000000000)

session_id = is_session_alive.count_changes().add_metric("sessionId")

navigation_id = input.page_id.count_changes().add_metric("navId")

subscope = make_tuple(session_id, navigation_id).count_changes()
subscope.add_metric("subscope_id")
print_ir_to_stdout()