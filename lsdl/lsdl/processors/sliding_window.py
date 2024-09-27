from typing import final

from ..lsp_model.component_base import BuiltinProcessorComponentBase
from ..lsp_model.core import SignalBase
from ..rust_code import RUST_DEFAULT_VALUE


@final
class SlidingTimeWindow(BuiltinProcessorComponentBase):
    def __init__(self,
                 clock: SignalBase | list[SignalBase],
                 data: SignalBase | list[SignalBase],
                 **kwargs):
        if 'emit_fn' in kwargs:
            emit_fn = kwargs['emit_fn']
        else:
            raise "Need to provide a emit_fn"

        if 'time_window_size' in kwargs:
            time_window_size = kwargs['time_window_size']
        else:
            raise "Need to provide a time window size"
        rust_processor_name = self.__class__.__name__
        init_value = kwargs.get("init_value", RUST_DEFAULT_VALUE)
        super().__init__(
            name=rust_processor_name,
            node_decl=f"{rust_processor_name}::new({emit_fn}, {time_window_size}, {init_value})",
            upstreams=[clock, data]
        )


@final
class SlidingWindow(BuiltinProcessorComponentBase):
    def __init__(self,
                 clock: SignalBase | list[SignalBase],
                 data: SignalBase | list[SignalBase],
                 **kwargs):
        if 'emit_fn' in kwargs:
            emit_fn = kwargs['emit_fn']
        else:
            raise "Need to provide a emit_fn"
        rust_processor_name = self.__class__.__name__
        init_value = kwargs.get("init_value", RUST_DEFAULT_VALUE)
        window_size = kwargs.get("window_size", 1)
        super().__init__(
            name=rust_processor_name,
            node_decl=f"{rust_processor_name}::new({emit_fn}, {window_size}, {init_value})",
            upstreams=[clock, data]
        )
