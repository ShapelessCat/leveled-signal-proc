from ..componet_base import BuiltinProcessorComponentBase
from ..signal import SignalBase


class Accumulator(BuiltinProcessorComponentBase):
    def __init__(self,
                 control: SignalBase,
                 data: SignalBase,
                 init_val: str = "Default::default()",
                 filter_lambda: str = "|_| true",
                 type_name: str = "i32"):
        rust_processor_name = self.__class__.__name__
        dt = data.get_rust_type_name()
        ct = control.get_rust_type_name()
        node_decl = f"{rust_processor_name}::<{dt},{ct}, _>::with_event_filter({init_val}, {filter_lambda})"
        super().__init__(
            name=rust_processor_name,
            node_decl=node_decl,
            upstreams=[control, data]
        )
        self.annotate_type(type_name)
