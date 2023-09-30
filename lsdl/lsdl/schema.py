import json
from typing import Any
from lsdl.signal import LeveledSignalBase

class TypeBase(LeveledSignalBase):
    def __init__(self, rs_type: str):
        super().__init__()
        self._rust_type = rs_type
        self._reset_expr = None
    def get_rust_type_name(self) -> str:
        return self._rust_type
    def render_rust_const(self, val) -> str:
        raise NotImplementedError()

class CompilerInferredType(TypeBase):
    def __init__(self):
        super().__init__("_")

class DateTime(TypeBase):
    def __init__(self, timezone: str = "Utc"):
        super().__init__("DateTime<chrono::" + timezone + ">")
    def render_rust_const(self, val) -> str:
        raise "Date time const value is not supported"

class String(TypeBase):
    def __init__(self):
        super().__init__("String")
    def render_rust_const(self, val) -> str:
        return json.dumps(val)

class Bool(TypeBase):
    def __init__(self):
        super().__init__("bool")
    def render_rust_const(self, val) -> str:
        return "true" if val else "false"

class Integer(TypeBase):
    def __init__(self, signed = True, width = 32):
        super().__init__("i" + str(width) if signed else "u" + str(width))
    def render_rust_const(self, val) -> str:
        return str(val) + self.get_rust_type_name()

class Float(TypeBase):
    def __init__(self, width = 64):
        super().__init__("f" + str(width))
    def render_rust_const(self, val) -> str:
        return str(val) + self.get_rust_type_name()

class Vector(TypeBase):
    def __init__(self, inner: TypeBase):
        super().__init__("Vec<" + inner.get_rust_type_name() + ">")
        self._inner = inner
    def render_rust_const(self, val) -> str:
        return "vec![{inner}]".format(
            inner = ",".join([self._inner.render_rust_const(v) for v in val])
        )

class InputMemberType(TypeBase):
    def __init__(self, inner: TypeBase):
        super().__init__(rs_type = inner.get_rust_type_name())
        self._inner = inner
        self._name = ""
    def set_name(self, name: str):
        self._name  = name
    def get_name(self) -> str:
        return self._name
    def get_id(self):
        return {
            "type": "InputSignal",
            "id": self.get_name(),
        }

class ClockCompanion(InputMemberType):
    def __init__(self):
        super().__init__(Integer(signed = False, width = 64))

class MappedInputType(InputMemberType):
    def __init__(self, input_key: str, inner: TypeBase):
        super().__init__(inner)
        self._input_key = input_key
        self._inner = inner
        self._reset_expr = self._inner._reset_expr
    def get_input_key(self) -> str:
        return self._input_key
    def clock(self) -> ClockCompanion:
        ret = ClockCompanion()
        ret.set_name(self.get_name() + "_clock")
        return ret

_defined_schema = None

class InputSchemaBase(TypeBase):
    def __init__(self, name = "InputSignalBag"):
        global _defined_schema
        super().__init__(name)
        self._members = []
        if "_timestamp_key" not in self.__dir__():
            self._timestamp_key = "timestamp"
        for item_name in self.__dir__():
            item = self.__getattribute__(item_name)
            if isinstance(item, TypeBase):
                if not isinstance(item, InputMemberType):
                    item = MappedInputType(input_key = item_name, inner = item)
                item.set_name(item_name)
                self.__setattr__(item_name, item)
                self._members.append(item_name)
        _defined_schema = self
    def to_dict(self)  -> dict:
        ret = {
            "type_name": self.get_rust_type_name(),
            "patch_timestamp_key": self._timestamp_key,
            "members": {}
        }
        for member in self._members:
            member_type = self.__getattribute__(member)
            ret["members"][member] = {
                "type": member_type.get_rust_type_name(),
                "clock_companion": member_type.clock().get_name(),
                "input_key": member_type.get_input_key(),
                "debug_info": member_type._debug_info.to_dict(),
            }
            if member_type._reset_expr is not None:
                ret["members"][member]["signal_behavior"] = {
                    "name": "Reset",
                    "default_expr": member_type._reset_expr,
                }
        return ret
    def get_id(self):
        return {
            "type": "InputBag",
        }
    
class SessionizedInputSchemaBase(InputSchemaBase):
    def create_session_signal(self) -> LeveledSignalBase:
        raise NotImplemented()
    def create_epoch_signal(self) -> LeveledSignalBase:
        raise NotImplemented()
    def _make_sessionized_input(self, key) -> LeveledSignalBase:
        if key not in self._sessionized_signals:
            raw_signal = super().__getattribute__(key)
            raw_signal_clock = raw_signal.clock()
            default_value = getattr(self, key + "_default", None)
            self._sessionized_signals[key] = self.sessionized(raw_signal, raw_signal_clock, default_value)
        return self._sessionized_signals[key]
    def sessionized(self, signal, signal_clock = None, default_value = None):
        if signal_clock is None:
            signal_clock = signal.clock()
        return self._scope_ctx.scoped(data = signal, clock = signal_clock, default = default_value)
    def __init__(self, name="InputSignalBag"):
        from lsdl.modules import ScopeContext
        super().__init__(name)
        self.session_signal = self.create_session_signal()
        self.epoch_signal = self.create_epoch_signal()
        self._sessionized_signals = dict()
        self._scope_ctx = ScopeContext(scope_level = self.session_signal, epoch = self.epoch_signal)
    def __getattribute__(self, name: str) -> Any:
        sessionized_prefix = "sessionized_"
        if name.startswith(sessionized_prefix):
            actual_key = name[len(sessionized_prefix):]
            return self._make_sessionized_input(actual_key)
        else:
            return super().__getattribute__(name)

def named(name: str, inner: TypeBase) -> TypeBase:
    return MappedInputType(name, inner)

def volatile(inner: TypeBase, default = None) -> TypeBase:
    if default is None:
        default = "Default::default()"
    inner._reset_expr = default
    return inner

def get_schema():
    return _defined_schema