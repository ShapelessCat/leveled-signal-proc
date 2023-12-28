from typing import Optional

from ..componet_base import BuiltinMeasurementComponentBase
from ..rust_code import RUST_DEFAULT_VALUE
from ..signal import SignalBase


class Peek(BuiltinMeasurementComponentBase):
    def __init__(self, input_signal: SignalBase):
        rule_component_name = self.__class__.__name__
        super().__init__(
            name=rule_component_name,
            node_decl=f"{rule_component_name}::default()",
            upstreams=[input_signal]
        )
        self.annotate_type(input_signal.get_rust_type_name())


class PeekTimestamp(BuiltinMeasurementComponentBase):
    def __init__(self, input_signal: SignalBase, closure: Optional[str] = None):
        is_mapped = closure is not None
        prefix = "Mapped" if is_mapped else ""
        rust_component_name = f"{prefix}{self.__class__.__name__}"

        if is_mapped:
            super().__init__(
                name=rust_component_name,
                node_decl=f"{rust_component_name}::new({closure}, {RUST_DEFAULT_VALUE})",
                upstreams=[input_signal]
            )
        else:
            super().__init__(
                name=rust_component_name,
                node_decl=f"{rust_component_name}::default()",
                upstreams=[input_signal]
            )
        self.annotate_type("u64")

