from ..componet_base import DirectBuiltinMeasurementComponentBase
from ..signal import SignalBase


class LinearChange(DirectBuiltinMeasurementComponentBase):
    def __init__(self, input_signal: SignalBase):
        rust_component_name = self.__class__.__name__
        super().__init__(
            name=rust_component_name,
            node_decl=f"{rust_component_name}::default()",
            upstreams=[input_signal]
        )
        self.annotate_type("f64")
