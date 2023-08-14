from lsdl.componet_base import BuiltinComponentBase
from lsdl.signal import LeveledSignalBase

class Accumulator(BuiltinComponentBase):
    def __init__(self, control: LeveledSignalBase, data: LeveledSignalBase, init_val = None, filter_lambda = None):
        if filter_lambda is None:
            filter_lambda = "|_| true"
        if init_val is None:
            init_val = "Default::default()"
        node_decl = "Accumulator::<{dt},{ct}, _>::with_event_filter({init_val}, {filter_lambda})".format(
            dt = data.get_rust_type_name(),
            ct = control.get_rust_type_name(),
            init_val = init_val,
            filter_lambda = filter_lambda
        )
        super().__init__(
            name = "Accumulator",
            is_measurement = False,
            node_decl = node_decl,
            upstreams = [control, data]
        )
