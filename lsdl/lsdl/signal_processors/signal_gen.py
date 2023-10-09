from lsdl.componet_base import BuiltinComponentBase


#    let a = SignalGenerator::square_wave(60_000_000_000, 0);
class SquareWave(BuiltinComponentBase):
    def __init__(self, period, phase = 0): 
        from lsdl.modules import _normalize_duration
        period = _normalize_duration(period)
        super().__init__(
            name = "SignalGenerator",
            is_measurement = False, 
            node_decl = f"SignalGenerator::square_wave({period}, {phase})", 
            upstreams = []
        )
        self.annotate_type("bool")

#    let b = SignalGenerator::raising_level(0, 2, 60_000_000_000, 0);
class MonotonicSteps(BuiltinComponentBase):
    def __init__(self, period, start = 0, step = 1, phase = 0):
        from lsdl.modules import _normalize_duration
        period = _normalize_duration(period)
        super().__init__(
            name = "SignalGenerator",
            is_measurement = False, 
            node_decl = f"SignalGenerator::raising_level({start}, {step}, {period}, {phase})", 
            upstreams = []
        )
        self.annotate_type("f64")

#    let c = SignalGenerator::<_, f64>::new(|ts| ((ts as f64).sin(), ts + 10));
class SignalGenerator(BuiltinComponentBase):
    def __init__(self, lambda_src, bind_var = "timestamp"):
        super().__init__(
            name = "SignalGenerator",
            is_measurement = False, 
            node_decl = f"SignalGenerator::new(|{bind_var}|{lambda_src})",
            upstreams = []
        )
