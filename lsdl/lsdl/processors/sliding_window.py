from typing import final

from ..lsp_model.component_base import BuiltinProcessorComponentBase
from ..lsp_model.core import SignalBase
from ..lsp_model.internal import normalize_duration
from ..rust_code import RUST_DEFAULT_VALUE, RustCode


@final
class SlidingTimeWindow(BuiltinProcessorComponentBase):
    def __init__(
        self,
        clock: SignalBase | list[SignalBase],
        data: SignalBase | list[SignalBase],
        emit_fn: RustCode,
        duration: int | str,
        init_value: RustCode = RUST_DEFAULT_VALUE,
    ):
        rust_processor_name = self.__class__.__name__
        time_window = normalize_duration(duration)
        super().__init__(
            name=rust_processor_name,
            node_decl=f"{rust_processor_name}::new({emit_fn}, {time_window}, {init_value})",
            upstreams=[clock, data],
        )


@final
class SlidingWindow(BuiltinProcessorComponentBase):
    def __init__(
        self,
        clock: SignalBase | list[SignalBase],
        data: SignalBase | list[SignalBase],
        emit_fn: RustCode,
        window_size: int = 1,
        init_value: RustCode = RUST_DEFAULT_VALUE,
    ):
        rust_processor_name = self.__class__.__name__
        super().__init__(
            name=rust_processor_name,
            node_decl=f"{rust_processor_name}::new({emit_fn}, {window_size}, {init_value})",
            upstreams=[clock, data],
        )
