from typing import final

from ..lsp_model.componet_base import DirectBuiltinMeasurementComponentBase
from ..lsp_model.core import SignalBase


@final
class LinearChange(DirectBuiltinMeasurementComponentBase):
    def __init__(self, input_signal: SignalBase):
        rust_component_name = self.__class__.__name__
        super().__init__(
            name=rust_component_name,
            node_decl=f"{rust_component_name}::default()",
            upstreams=[input_signal]
        )
        self.annotate_type("f64")