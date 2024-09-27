from typing import final

from ..lsp_model.component_base import BuiltinProcessorComponentBase
from ..lsp_model.core import SignalBase
from ..rust_code import RUST_DEFAULT_VALUE, RustCode


@final
class Accumulator(BuiltinProcessorComponentBase):
    def __init__(
        self,
        control: SignalBase,
        data: SignalBase,
        init_val: RustCode = RUST_DEFAULT_VALUE,
        filter_lambda: RustCode = "|_| true",
        type_name: RustCode = "i32",
    ):
        rust_processor_name = self.__class__.__name__
        dt = data.get_rust_type_name()
        ct = control.get_rust_type_name()
        node_decl = f"""
            {rust_processor_name}::<{dt},{ct}, _>::with_event_filter(
                {init_val}, {filter_lambda}
            )
        """
        super().__init__(
            name=rust_processor_name, node_decl=node_decl, upstreams=[control, data]
        )
        self.annotate_type(type_name)
