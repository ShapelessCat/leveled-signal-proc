from ..componet_base import DirectBuiltinMeasurementComponentBase
from ..signal import SignalBase


class Peek(DirectBuiltinMeasurementComponentBase):
    def __init__(self, input_signal: SignalBase):
        rule_component_name = self.__class__.__name__
        super().__init__(
            name=rule_component_name,
            node_decl=f"{rule_component_name}::default()",
            upstreams=[input_signal]
        )
        self.annotate_type(input_signal.get_rust_type_name())


class PeekTimestamp(DirectBuiltinMeasurementComponentBase):
    def __init__(self, input_signal: SignalBase):
        rust_component_name = self.__class__.__name__
        super().__init__(
            name=rust_component_name,
            node_decl=f"{rust_component_name}",
            upstreams=[input_signal]
        )
        self.annotate_type("u64")
