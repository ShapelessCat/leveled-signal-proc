from typing import final

from lsdl.rust_code import RustPrimitiveType

from ..lsp_model.component_base import DirectBuiltinMeasurementComponentBase
from ..lsp_model.core import SignalBase


@final
class DurationTrue(DirectBuiltinMeasurementComponentBase):
    def __init__(self, input_signal: SignalBase):
        rust_component_name = self.__class__.__name__
        super().__init__(
            name=rust_component_name,
            node_decl=f"{rust_component_name}::default()",
            upstreams=[input_signal],
        )
        self.annotate_type(RustPrimitiveType.U64.value)


@final
class DurationSinceBecomeTrue(DirectBuiltinMeasurementComponentBase):
    def __init__(self, input_signal: SignalBase):
        rust_component_name = self.__class__.__name__
        super().__init__(
            name=rust_component_name,
            node_decl=f"{rust_component_name}::default()",
            upstreams=[input_signal],
        )
        self.annotate_type(RustPrimitiveType.U64.value)


@final
class DurationOfCurrentLevel(DirectBuiltinMeasurementComponentBase):
    def __init__(self, input_signal: SignalBase):
        rust_component_name = self.__class__.__name__
        super().__init__(
            name=rust_component_name,
            node_decl=f"{rust_component_name}::default()",
            upstreams=[input_signal],
        )
        self.annotate_type(RustPrimitiveType.U64.value)
