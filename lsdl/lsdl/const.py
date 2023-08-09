from lsdl.schema import Bool, Float, Integer, String, TypeBase
from lsdl.signal import LeveledSignalBase

class Const(LeveledSignalBase):
    def __init__(self, value, val_type: TypeBase = None):
        if val_type is None:
            if type(value) == int:
                val_type = Integer()
            elif type(value) == str:
                val_type = String()
            elif type(value) == float:
                val_type = Float()
            elif type(value) == bool:
                val_type = Bool()
        self._type = val_type
        self._rs_value = val_type.render_rust_const(value)
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