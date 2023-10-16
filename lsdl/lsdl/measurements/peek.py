from ..componet_base import BuiltinMeasurementComponentBase
from ..signal import LeveledSignalProcessingModelComponentBase


class PeekValue(BuiltinMeasurementComponentBase):
    def __init__(self, input_signal: LeveledSignalProcessingModelComponentBase):
        super().__init__(
            name = "Peek",
            node_decl = "Peek::default()",
            upstreams = [input_signal]
        )
        self.annotate_type(input_signal.get_rust_type_name())
