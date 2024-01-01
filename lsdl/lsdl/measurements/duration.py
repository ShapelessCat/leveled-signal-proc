from ..componet_base import DirectBuiltinMeasurementComponentBase
from ..signal import SignalBase


# TODO: Fix this later when adding direct measurement combinator support is done.
class DurationTrue(DirectBuiltinMeasurementComponentBase):
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
        self.annotate_type("u64")


class DurationSinceBecomeTrue(DirectBuiltinMeasurementComponentBase):
    def __init__(self, input_signal: SignalBase):
        rust_component_name = self.__class__.__name__
        super().__init__(
            name=rust_component_name,
            node_decl=f"{rust_component_name}::default()",
            upstreams=[input_signal]
        )
        self.annotate_type("u64")


class DurationSinceLastLevel(DirectBuiltinMeasurementComponentBase):
    def __init__(self, input_signal: SignalBase):
        rust_component_name = self.__class__.__name__
        super().__init__(
            name=rust_component_name,
            node_decl=f"{rust_component_name}::default()",
            upstreams=[input_signal]
        )
        self.annotate_type("u64")
