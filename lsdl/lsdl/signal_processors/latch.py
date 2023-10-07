from lsdl.componet_base import BuiltinComponentBase
from lsdl.signal import LeveledSignalBase

class Latch(BuiltinComponentBase):
    def __init__(self, control: LeveledSignalBase, data: LeveledSignalBase, forget_duration = -1, **kwargs):
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
        if "output_type" in kwargs:
            self.annotate_type(kwargs["output_type"])
        else:
            self.annotate_type(data.get_rust_type_name())

class EdgeTriggeredLatch(BuiltinComponentBase):
    def __init__(self, control: LeveledSignalBase, data: LeveledSignalBase, forget_duration = -1, **kwargs):
        from lsdl.modules import _normalize_duration
        forget_duration = _normalize_duration(forget_duration)
        if forget_duration < 0:
            node_decl = "EdgeTriggeredLatch::<{control_type_name}, {data_type_name}>::default()".format(control_type_name = control.get_rust_type_name(), data_type_name = data.get_rust_type_name())
        else:
            node_decl = "EdgeTriggeredLatch::with_forget_behavior(<{type_name} as Default>::default(), <{type_name} as Default>::default(), {forget_duration})".format(
                type_name = data.get_rust_type_name(),
                forget_duration = forget_duration
            )
        super().__init__(
            name = "EdgeTriggeredLatch",
            is_measurement = False, 
            node_decl = node_decl, 
            upstreams = [control, data]
        )
        if "output_type" in kwargs:
            self.annotate_type(kwargs["output_type"])
        else:
            self.annotate_type(data.get_rust_type_name())