from typing import final

from ...lsp_model.component_base import IndirectBuiltinMeasurementComponentBase
from ...lsp_model.core import MeasurementBase


@final
class MappedMeasurement(IndirectBuiltinMeasurementComponentBase):
    def __init__(self, bind_var: str, lambda_src: str, inner: MeasurementBase):
        rust_component_name = self.__class__.__name__
        super().__init__(
            name=rust_component_name,
            upstreams=[inner],
            node_decl=f"""
                {rust_component_name}::new(
                    |{bind_var}| {lambda_src},
                    {self.get_id_or_literal_value(inner)}.clone()
                )
            """,
        )
