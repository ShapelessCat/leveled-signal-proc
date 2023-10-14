from ..componet_base import BuiltinMeasurementComponentBase
from ..signal import LeveledSignalBase


class DiffSinceCurrentLevel(BuiltinMeasurementComponentBase):
    def __init__(self, control: LeveledSignalBase, data: LeveledSignalBase):
        super().__init__(
            name = "DiffSinceCurrentLevel",
            is_measurement = True,
            node_decl = "DiffSinceCurrentLevel::default()",
            upstreams = [control, data],
        )
        self.annotate_type(data.get_rust_type_name())