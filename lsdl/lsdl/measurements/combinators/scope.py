from ...componet_base import IndirectBuiltinMeasurementComponentBase
from ...measurement import MeasurementBase
from ...signal import SignalBase


class ScopedMeasurement(IndirectBuiltinMeasurementComponentBase):
    def __init__(self, scope_signal: SignalBase, inner: MeasurementBase):
        rust_component_name = self.__class__.__name__
        super().__init__(
            name=rust_component_name,
            upstreams=[scope_signal, inner],
            node_decl=f"{rust_component_name}::new({self.get_id_or_literal_value(inner)})",
        )
        self.annotate_type(inner.get_rust_type_name())
