from typing import final

from lsdl.rust_code import RustPrimitiveType

from ..lsp_model.component_base import BuiltinProcessorComponentBase
from ..lsp_model.core import SignalBase


@final
class LivenessChecker(BuiltinProcessorComponentBase):
    def __init__(
        self,
        liveness_clock: SignalBase,
        ef_bind_var: str,
        ef_src: str,
        timeout=90_000_000_000,
    ):
        rust_processor_name = self.__class__.__name__
        super().__init__(
            name=rust_processor_name,
            node_decl=f"""
                {rust_processor_name}::new(
                    |{ef_bind_var}: &InputSignalBagPatch| {ef_src}, {timeout}
                )
            """,
            upstreams=[liveness_clock],
        )
        self.annotate_type(RustPrimitiveType.BOOL.value)
