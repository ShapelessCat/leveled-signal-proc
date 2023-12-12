from ..componet_base import BuiltinMeasurementComponentBase
from ..signal import SignalBase


class LinearChange(BuiltinMeasurementComponentBase):
    def __init__(self, input_signal: SignalBase, scope_signal=None):
        is_scoped = scope_signal is not None
        prefix = "Scoped" if is_scoped else ""
        rust_component_name = f"{prefix}{self.__class__.__name__}"
        upstreams = [scope_signal, input_signal] if is_scoped else [input_signal]
        super().__init__(
            name=rust_component_name,
            node_decl=f"{rust_component_name}::default()",
            upstreams=upstreams
        )
        self.annotate_type("f64")
