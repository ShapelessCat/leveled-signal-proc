from lsdl.schema import Bool, Float, Integer, String, TypeBase
from lsdl.signal import LeveledSignalBase


class Const(LeveledSignalBase):
    """Constant value signal."""
    def __init__(self, value, val_type: TypeBase = None):
        super().__init__()
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
        self._type = val_type
        self._rs_value = val_type.render_rust_const(value)

    def is_signal(self) -> bool:
        return True

    def get_rust_type_name(self) -> str:
        return self._type.get_rust_type_name()

    def get_rust_instant_value(self) -> str:
        return self._rs_value

    def get_id(self):
        return {
            "type": "Constant",
            "value": self._rs_value,
            "type_name": self._type.get_rust_type_name(),
        }
