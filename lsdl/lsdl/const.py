from typing import Optional

from .rust_code import RustCode
from .schema import Bool, Float, Integer, String, TypeWithLiteralValue
from .signal import SignalBase


class Const(SignalBase):
    """Constant value signal."""
    def __init__(self, value, val_type: Optional[TypeWithLiteralValue] = None):
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
        if val_type is None:
            raise Exception("Can't render this value to a Rust constant.")
        super().__init__(val_type.get_rust_type_name())
        self._rust_constant_value = val_type.render_rust_const(value)

    @property
    def rust_constant_value(self) -> RustCode:
        return self._rust_constant_value

    def get_id(self):
        return {
            "type": "Constant",
            "value": self.rust_constant_value,
            "type_name": self.get_rust_type_name(),
        }
