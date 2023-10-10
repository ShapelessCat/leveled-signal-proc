from ..componet_base import BuiltinComponentBase
from ..signal import LeveledSignalBase


class PeekValue(BuiltinComponentBase):
    def __init__(self, input_signal: LeveledSignalBase):
        super().__init__(
            name = "Peek",
            is_measurement = True,
            node_decl = "Peek::default()",
            upstreams = [input_signal]
        )
        self.annotate_type(input_signal.get_rust_type_name())
