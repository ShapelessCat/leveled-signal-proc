from lsdl.componet_base import BuiltinComponentBase
from lsdl.signal import LeveledSignalBase

class Latch(BuiltinComponentBase):
    def __init__(self, control: LeveledSignalBase, data: LeveledSignalBase, forget_duration = -1):
        if forget_duration < 0:
            node_decl = "Latch::<{type_name}>::default()".format(type_name = data.get_rust_type_name())
        else:
            node_decl = "Latch::with_forget_behavior(<{type_name} as Default>::default(), <{type_name} as Default>::default(), {forget_duration})".format(
                type_name = data.get_rust_type_name(),
                forget_duration = forget_duration
            )
        super().__init__(
            name = "Latch",
            is_measurement = False, 
            node_decl = node_decl, 
            upstreams = [control, data]
        )
