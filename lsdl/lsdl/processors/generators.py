from typing import Optional, final

from ..lsp_model.component_base import BuiltinProcessorComponentBase
from ..lsp_model.core import SignalBase
from ..lsp_model.schema import (
    Bool,
    Float,
    Integer,
    LspEnumBase,
    String,
    TypeWithLiteralValue,
)
from ..rust_code import RustCode


@final
class Const(SignalBase):
    """Constant value signal."""

    def __init__(
        self,
        value,
        need_owned: bool = True,
        val_type: Optional[TypeWithLiteralValue] = None,
    ):
        if isinstance(value, LspEnumBase):
            super().__init__(value.__class__.__name__)
            self._rust_constant_value = str(value)
        else:
            if val_type is None:
                tpe = type(value)
                if tpe == int:
                    val_type = Integer()
                elif tpe == str:
                    val_type = String()
                elif tpe == float:
                    val_type = Float()
                elif tpe == bool:
                    val_type = Bool()
                elif tpe == list:
                    raise NotImplementedError("Not implemented yet")  # Till now no strong requirement
            if val_type is None:
                raise Exception("Can't render this value to a Rust constant.")
            super().__init__(val_type.get_rust_type_name())
            self._rust_constant_value = val_type.render_rust_const(value, need_owned)

    @property
    def rust_constant_value(self) -> RustCode:
        return self._rust_constant_value

    def get_description(self):
        return {
            "type": "Constant",
            "value": self.rust_constant_value,
            "type_name": self.get_rust_type_name(),
        }


_rust_component_name: RustCode = "SignalGenerator"


#    let a = SignalGenerator::square_wave(60_000_000_000, 0);
@final
class SquareWave(BuiltinProcessorComponentBase):
    def __init__(self, period, phase=0):
        from ..lsp_model.internal import normalize_duration

        period = normalize_duration(period)
        super().__init__(
            name=_rust_component_name,
            node_decl=f"{_rust_component_name}::square_wave({period}, {phase})",
            upstreams=[],
        )
        self.annotate_type("bool")


#    let b = SignalGenerator::raising_level(0, 2, 60_000_000_000, 0);
@final
class MonotonicSteps(BuiltinProcessorComponentBase):
    def __init__(self, period, start=0, step=1, phase=0):
        from ..lsp_model.internal import normalize_duration

        period = normalize_duration(period)
        super().__init__(
            name=_rust_component_name,
            node_decl=f"{_rust_component_name}::raising_level({start}, {step}, {period}, {phase})",
            upstreams=[],
        )
        self.annotate_type("f64")


#    let c = SignalGenerator::<_, f64>::new(|ts| ((ts as f64).sin(), ts + 10));
@final
class SignalGenerator(BuiltinProcessorComponentBase):
    def __init__(self, lambda_src, bind_var="timestamp"):
        super().__init__(
            name=_rust_component_name,
            node_decl=f"{_rust_component_name}::new(|{bind_var}| {lambda_src})",
            upstreams=[],
        )
