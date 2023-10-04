from schema import Input
from lsdl.signal_processors import EdgeTriggeredLatch
from lsdl import print_ir_to_stdout, measurement_config
from lsdl.const import Const
from lsdl.modules import make_tuple
from lsdl.signal import If
from lsdl.signal_processors.signal_gen import SquareWave

input = Input()

is_session_alive = EdgeTriggeredLatch(input.event_name.clock(), data = Const(True), forget_duration = 90000000000)

session_id = is_session_alive.count_changes().add_metric("sessionId")

navigation_id = input.page_id.count_changes().add_metric("navId")

subscope = make_tuple(session_id, navigation_id).count_changes()
subscope.add_metric("subscope_id")

measurement_config()\
    .disable_measure_for_event()\
    .set_trigger_signal(make_tuple(SquareWave("60s"), is_session_alive))

print_ir_to_stdout()