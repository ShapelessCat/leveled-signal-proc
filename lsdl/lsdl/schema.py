import json
from abc import ABC, abstractmethod
from typing import Any, Optional

from .signal import LeveledSignalProcessingModelComponentBase, SignalBase


class TypeBase(LeveledSignalProcessingModelComponentBase, ABC):
    def __init__(self, rs_type: str):
        super().__init__()
        self._rust_type = rs_type
        self._reset_expr = None
        self._parent = None

    @property
    def reset_expr(self):
        return self._reset_expr

    def get_rust_type_name(self) -> str:
        return self._rust_type

    def get_id(self):
        try:
            return super().get_id()
        except Exception as e:
            if self._parent is not None:
                return self._parent.get_id()
            raise e


class TypeWithLiteralValue(TypeBase, ABC):
    @abstractmethod
    def render_rust_const(self, val) -> str:
        raise NotImplementedError()


class CompilerInferredType(TypeBase):
    def __init__(self):
        super().__init__("_")


class DateTime(TypeBase):
    def __init__(self, timezone: str = "Utc"):
        super().__init__("chrono::DateTime<chrono::" + timezone + ">")


class String(TypeWithLiteralValue):
    def __init__(self):
        super().__init__("String")

    # @override
    def render_rust_const(self, val) -> str:
        return f"{json.dumps(val)}.to_string()"


class Bool(TypeWithLiteralValue):
    def __init__(self):
        super().__init__("bool")

    # @override
    def render_rust_const(self, val) -> str:
        return "true" if val else "false"


class Integer(TypeWithLiteralValue):
    def __init__(self, signed=True, width=32):
        super().__init__("i" + str(width) if signed else "u" + str(width))

    # @override
    def render_rust_const(self, val) -> str:
        return str(val) + self.get_rust_type_name()


class Float(TypeWithLiteralValue):
    def __init__(self, width=64):
        super().__init__("f" + str(width))

    # @override
    def render_rust_const(self, val) -> str:
        return str(val) + self.get_rust_type_name()


class Vector(TypeWithLiteralValue):
    def __init__(self, element_type: TypeBase):
        super().__init__("Vec<" + element_type.get_rust_type_name() + ">")
        self._element_type = element_type

    # @override
    def render_rust_const(self, val) -> str:
        typed_const_elements = ",".join([self._element_type.render_rust_const(v) for v in val])
        return f"vec![{typed_const_elements}]"


class InputMember(SignalBase, ABC):
    def __init__(self, tpe: TypeBase):
        super().__init__()
        self._rust_type = tpe.get_rust_type_name()
        self._reset_expr = None
        self._parent = None
        self._inner = tpe
        tpe._parent = self
        self._name = ""

    @property
    def name(self) -> str:
        return self._name

    @name.setter
    def name(self, name: str):
        self._name = name

    def get_rust_type_name(self) -> str:
        return self._rust_type

    def get_id(self):
        return {
            "type": "InputSignal",
            "id": self.name,
        }


class ClockCompanion(InputMember):
    def __init__(self):
        super().__init__(Integer(signed=False, width=64))


class MappedInputMember(InputMember):
    def __init__(self, input_key: str, tpe: TypeBase):
        super().__init__(tpe)
        self._input_key = input_key
        self._inner = tpe
        self._reset_expr = self._inner.reset_expr

    @property
    def reset_expr(self):
        return self._reset_expr

    def get_input_key(self) -> str:
        return self._input_key

    def clock(self) -> ClockCompanion:
        ret = ClockCompanion()
        ret.name = self.name + "_clock"
        return ret

    # TODO: move this outside of this class or convert them to static methods!!!
    def parse(self, type_name, default_value="Default::default()") -> SignalBase:
        return self \
            .map(
                bind_var="s",
                lambda_src=f"s.parse::<{type_name}>().unwrap_or({default_value})"
            ).annotate_type(type_name)

    def starts_with(self, other) -> SignalBase:
        from .const import Const
        from .modules import make_tuple
        if not isinstance(other, LeveledSignalProcessingModelComponentBase):
            other = Const(other)
        return make_tuple(self, other).map(
            bind_var="(s, p)",
            lambda_src="s.starts_with(p)"
        ).annotate_type("bool")

    def timestamp(self):
        return self.map(
            bind_var="t",
            lambda_src="t.timestamp()"
        ).annotate_type("i64")


_defined_schema: Optional['InputSchemaBase'] = None


class InputSchemaBase(LeveledSignalProcessingModelComponentBase):
    def __init__(self, name="InputSignalBag"):
        global _defined_schema
        super().__init__()
        self._rust_type = name
        self._members = []
        if "_timestamp_key" not in self.__dir__():
            self._timestamp_key = "timestamp"
        for item_name in self.__dir__():
            item = self.__getattribute__(item_name)
            if isinstance(item, (TypeBase, InputMember)):
                if isinstance(item, TypeBase):
                    item = MappedInputMember(input_key=item_name, tpe=item)
                item.name = item_name
                self.__setattr__(item_name, item)
                self._members.append(item_name)
        _defined_schema = self

    def rebuild(self, name="InputSignalBag"):
        self.__init__(name)

    def get_rust_type_name(self) -> str:
        return self._rust_type

    def to_dict(self) -> dict:
        ret = {
            "type_name": self.get_rust_type_name(),
            "patch_timestamp_key": self._timestamp_key,
            "members": {}
        }
        for member in self._members:
            member_type = getattr(self, member)
            ret["members"][member] = {
                "type": member_type.get_rust_type_name(),
                "clock_companion": member_type.clock().name,
                "input_key": member_type.get_input_key(),
                "debug_info": member_type.debug_info.to_dict(),
            }
            if member_type.reset_expr is not None:
                ret["members"][member]["signal_behavior"] = {
                    "name": "Reset",
                    "default_expr": member_type.reset_expr,
                }
        return ret

    def get_id(self):
        return {
            "type": "InputBag",
        }


class SessionizedInputSchemaBase(InputSchemaBase):
    def create_session_signal(self) -> SignalBase:
        raise NotImplemented()

    def create_epoch_signal(self) -> SignalBase:
        raise NotImplemented()

    def _make_sessionized_input(self, key) -> SignalBase:
        if key not in self._sessionized_signals:
            raw_signal = super().__getattribute__(key)
            raw_signal_clock = raw_signal.clock()
            default_value = getattr(self, key + "_default", None)
            self._sessionized_signals[key] = self.sessionized(raw_signal, raw_signal_clock, default_value)
        return self._sessionized_signals[key]

    def sessionized(self, signal, signal_clock=None, default_value=None):
        if signal_clock is None:
            signal_clock = signal.clock()
        return self._scope_ctx.scoped(data=signal, clock=signal_clock, default=default_value)

    def __init__(self, name="InputSignalBag"):
        from .modules import ScopeContext
        super().__init__(name)
        self.session_signal = self.create_session_signal()
        self.epoch_signal = self.create_epoch_signal()
        self._sessionized_signals = dict()
        self._scope_ctx = ScopeContext(scope_level=self.session_signal, epoch=self.epoch_signal)

    def __getattribute__(self, name: str) -> Any:
        sessionized_prefix = "sessionized_"
        if name.startswith(sessionized_prefix):
            actual_key = name[len(sessionized_prefix):]
            return self._make_sessionized_input(actual_key)
        else:
            return super().__getattribute__(name)


def named(name: str, inner: TypeBase = String()) -> MappedInputMember:
    return MappedInputMember(name, inner)


def volatile(inner: TypeBase, default=None) -> TypeBase:
    if default is None:
        default = "Default::default()"
    inner._reset_expr = default
    return inner


def get_schema():
    return _defined_schema


def create_type_model_from_rust_type_name(rust_type_name: str) -> Optional[TypeBase]:
    if rust_type_name == "String":
        return String()
    if rust_type_name[0] in ['i', 'u']:
        width = int(rust_type_name[1:])
        signed = rust_type_name[0] == 'i'
        return Integer(signed, width)
    if rust_type_name[0] == 'f':
        width = int(rust_type_name[1:])
        return Float(width)
    if rust_type_name[0] == 'bool':
        return Bool()
    return None
