from ..componet_base import BuiltinMeasurementComponentBase
from ..signal import LeveledSignalProcessingModelComponentBase


class DiffSinceCurrentLevel(BuiltinMeasurementComponentBase):
    def __init__(self, control: LeveledSignalProcessingModelComponentBase, data: LeveledSignalProcessingModelComponentBase):
        super().__init__(
            name = "DiffSinceCurrentLevel",
            node_decl = "DiffSinceCurrentLevel::default()",
            upstreams = [control, data],
        )
        self.annotate_type(data.get_rust_type_name())