from typing import final
from ...componet_base import IndirectBuiltinMeasurementComponentBase
from ...measurement import MeasurementBase
from ...rust_code import RustCode


@final
class BinaryCombinedMeasurement(IndirectBuiltinMeasurementComponentBase):
    def __init__(self,
                 bind_var0: RustCode, bind_var1: RustCode, lambda_src: RustCode,
                 inner0: MeasurementBase,
                 inner1: MeasurementBase):
        rust_component_name = self.__class__.__name__
        super().__init__(
            name=rust_component_name,
            upstreams=[inner0, inner1],
            node_decl=f"""
                {rust_component_name}::new(
                    |{bind_var0}, {bind_var1}| {lambda_src},
                    {self.get_id_or_literal_value(inner0)}.clone(),
                    {self.get_id_or_literal_value(inner1)}.clone()
                )
            """,
        )
