from ..componet_base import BuiltinProcessorComponentBase
from ..rust_code import RustCode

_rust_component_name: RustCode = "SignalGenerator"


#    let a = SignalGenerator::square_wave(60_000_000_000, 0);
class SquareWave(BuiltinProcessorComponentBase):
    def __init__(self, period, phase=0):
        from ..modules import normalize_duration
        period = normalize_duration(period)
        super().__init__(
            name=_rust_component_name,
            node_decl=f"{_rust_component_name}::square_wave({period}, {phase})",
            upstreams=[]
        )
        self.annotate_type("bool")


#    let b = SignalGenerator::raising_level(0, 2, 60_000_000_000, 0);
class MonotonicSteps(BuiltinProcessorComponentBase):
    def __init__(self, period, start=0, step=1, phase=0):
        from ..modules import normalize_duration
        period = normalize_duration(period)
        super().__init__(
            name=_rust_component_name,
            node_decl=f"{_rust_component_name}::raising_level({start}, {step}, {period}, {phase})",
            upstreams=[]
        )
        self.annotate_type("f64")


#    let c = SignalGenerator::<_, f64>::new(|ts| ((ts as f64).sin(), ts + 10));
class SignalGenerator(BuiltinProcessorComponentBase):
    def __init__(self, lambda_src, bind_var="timestamp"):
        super().__init__(
            name=_rust_component_name,
            node_decl=f"{_rust_component_name}::new(|{bind_var}| {lambda_src})",
            upstreams=[]
        )
