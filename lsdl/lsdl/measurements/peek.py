from lsdl.componet_base import BuiltinComponentBase
from lsdl.signal import LeveledSignalBase

class PeekValue(BuiltinComponentBase):
    def __init__(self, input: LeveledSignalBase):
        super().__init__(
            name = "Peek",
            is_measurement = True,
            node_decl = "Peek::default()",
            upstreams = [input]
        )
        self._output_type = input.get_rust_type_name()

