from lsdl.componet_base import BuiltinComponentBase
from lsdl.signal import LeveledSignalBase

class DurationTrue(BuiltinComponentBase):
    def __init__(self, input: LeveledSignalBase):
        super().__init__(
            name = "DurationTrue",
            is_measurement = True,
            node_decl = "DurationTrue::default()",
            upstreams = [input]
        )
