from ..componet_base import BuiltinMeasurementComponentBase
from ..signal import LeveledSignalBase


class DurationTrue(BuiltinMeasurementComponentBase):
    def __init__(self, input_signal: LeveledSignalBase, scope_signal = None):
        if scope_signal is None:
            super().__init__(
                name = "DurationTrue",
                node_decl = "DurationTrue::default()",
                upstreams = [input_signal]
            )
        else:
            super().__init__(
                name = "ScopedDurationTrue",
                node_decl = "ScopedDurationTrue::default()",
                upstreams = [scope_signal, input_signal]
            )
        self.annotate_type("u64")


class DurationSinceBecomeTrue(BuiltinMeasurementComponentBase):
    def __init__(self, input_signal: LeveledSignalBase):
        super().__init__(
            name = "DurationSinceBecomeTrue",
            node_decl = "DurationSinceBecomeTrue::default()",
            upstreams = [input_signal]
        )
        self.annotate_type("u64")
