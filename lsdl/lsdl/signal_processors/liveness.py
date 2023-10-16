from ..componet_base import BuiltinProcessorComponentBase
from ..signal import LeveledSignalProcessingModelComponentBase


class LivenessChecker(BuiltinProcessorComponentBase):
    def __init__(self, liveness_clock: LeveledSignalProcessingModelComponentBase, ef_bind_var: str, ef_src: str, timeout = 90_000_000_000):
        node_decl = """LivenessChecker::new(
            |{var}: &InputSignalBagPatch| {code},
            {timeout},
        )""".format(
            var = ef_bind_var,
            code = ef_src,
            timeout = timeout
        )
        super().__init__(
            name = "LivenessChecker",
            node_decl = node_decl,
            upstreams = [liveness_clock],
        )
        self.annotate_type("bool")
