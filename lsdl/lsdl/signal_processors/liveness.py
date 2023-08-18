from lsdl.componet_base import BuiltinComponentBase
from lsdl.signal import LeveledSignalBase

class LivenessChecker(BuiltinComponentBase):
    def __init__(self, liveness_clock: LeveledSignalBase, ef_bind_var:str, ef_src: str, timeout = 90_000_000_000):
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
            is_measurement = False, 
            node_decl = node_decl, 
            upstreams = [liveness_clock],
        )
        self._output_type = "bool"