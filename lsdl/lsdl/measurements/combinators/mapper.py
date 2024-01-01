from lsdl.measurement import MeasurementBase

from ...componet_base import IndirectBuiltinMeasurementComponentBase

class MappedMeasurement(IndirectBuiltinMeasurementComponentBase):
    def __init__(self, bind_var: str, lambda_src: str, inner: MeasurementBase):
        rust_component_name = self.__class__.__name__
        super().__init__(
            name=rust_component_name,
            upstreams=[inner],
            node_decl=f"{rust_component_name}::new(|{bind_var}| {lambda_src }, {self.get_id_or_literal_value(inner)})",
        )
        # self.annotate_type(input_signal.get_rust_type_name())
