import json
from abc import ABC, abstractmethod
from typing import Optional, final

from .core import LeveledSignalProcessingModelComponentBase, SignalBase
from ..rust_code import INPUT_SIGNAL_BAG, RUST_DEFAULT_VALUE, RustCode


class _TypeBase(LeveledSignalProcessingModelComponentBase, ABC):
    def __init__(self, rust_type: RustCode):
        super().__init__(rust_type)
        self._reset_expr = None
        self._parent: Optional[_InputMember] = None

    @property
    def reset_expr(self):
        return self._reset_expr

    def get_description(self):
        try:
            return super().get_description()
        except Exception as e:
            if self._parent is not None:
                return self._parent.get_description()
            raise e


class TypeWithLiteralValue(_TypeBase, ABC):
    @abstractmethod
    def render_rust_const(self, val) -> RustCode:
        raise NotImplementedError()


@final
class DateTime(_TypeBase):
    def __init__(self, timezone: RustCode = "Utc"):
        super().__init__(f"chrono::DateTime<chrono::{timezone}>")


@final
class String(TypeWithLiteralValue):
    def __init__(self):
        super().__init__("String")

    # @override
    def render_rust_const(self, val) -> RustCode:
        return f"{json.dumps(val)}.to_string()"


@final
class Bool(TypeWithLiteralValue):
    def __init__(self):
        super().__init__("bool")

    # @override
    def render_rust_const(self, val) -> RustCode:
        return "true" if val else "false"


@final
class Integer(TypeWithLiteralValue):
    def __init__(self, signed=True, width=32):
        type_prefix = "i" if signed else "u"
        super().__init__(f"{type_prefix}{width}")

    # @override
    def render_rust_const(self, val) -> RustCode:
        return str(val) + self.get_rust_type_name()


@final
class Float(TypeWithLiteralValue):
    def __init__(self, width=64):
        super().__init__(f"f{width}")

    # @override
    def render_rust_const(self, val) -> RustCode:
        return str(val) + self.get_rust_type_name()


@final
class Vector(TypeWithLiteralValue):
    def __init__(self, element_type: _TypeBase):
        super().__init__(f"Vec<{element_type.get_rust_type_name()}>")
        self._element_type = element_type

    # @override
    def render_rust_const(self, val) -> RustCode:
        if isinstance(self._element_type, TypeWithLiteralValue):
            typed_const_elements = ", ".join([self._element_type.render_rust_const(v) for v in val])
            return f"vec![{typed_const_elements}]"
        else:
            raise Exception("Not a vector literal!")


class _InputMember(SignalBase, ABC):
    def __init__(self, tpe: _TypeBase, name=""):
        super().__init__(tpe.get_rust_type_name())
        tpe._parent = self
        self._inner = tpe
        self._name = name
        self._reset_expr = None

    @property
    def name(self) -> str:
        return self._name

    @name.setter
    def name(self, name: str):
        self._name = name

    def get_description(self):
        return {
            "type": "InputSignal",
            "id": self.name,
        }


@final
class _ClockCompanion(_InputMember):
    def __init__(self, name):
        super().__init__(Integer(signed=False, width=64), name)


@final
class MappedInputMember(_InputMember):
    def __init__(self, input_key: str, tpe: _TypeBase):
        super().__init__(tpe)
        self._input_key = input_key
        self._reset_expr = self._inner.reset_expr

    @property
    def reset_expr(self):
        return self._reset_expr

    def get_input_key(self) -> str:
        return self._input_key

    def clock(self) -> _ClockCompanion:
        # This `self.name` will be given when initializing the `InputSchemaBase` through reflection.
        return _ClockCompanion(f"{self.name}_clock")

    # TODO: move this outside of this class or convert them to static methods!!!
    def parse(self, type_name, default_value: RustCode = RUST_DEFAULT_VALUE) -> SignalBase:
        return self.map(
            bind_var="s",
            lambda_src=f"s.parse::<{type_name}>().unwrap_or({default_value})"
        ).annotate_type(type_name)

    def starts_with(self, other) -> SignalBase:
        from ..processors import Const, make_tuple
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


class InputSchemaBase(SignalBase):
    def __init__(self, type_name: RustCode = INPUT_SIGNAL_BAG):
        global _defined_schema
        self.type_name = type_name
        # If treating `InputSchemaBase` itself as a signal/clock, its type should be `u64`.
        # Actually, lsp-codegen will always automatically insert a `_clock: u64` field
        # to the codegen result struct of this class, and the generated struct name should
        # be the value of `self.type_name`.
        super().__init__("u64")
        self._members = []
        if "_timestamp_key" not in self.__dir__():
            self._timestamp_key = "timestamp"
        for item_name in self.__dir__():
            item = self.__getattribute__(item_name)
            # There won't be members as `ClockCompanion`s in the source code of
            # an `InputSchemaBase` instance, therefore we don't try to handle it here.
            if isinstance(item, _TypeBase):
                item = MappedInputMember(input_key=item_name, tpe=item)
            if isinstance(item, MappedInputMember):
                item.name = item_name
                self.__setattr__(item_name, item)
                self._members.append(item_name)
        _defined_schema = self

    def rebuild(self, rust_type: RustCode = INPUT_SIGNAL_BAG):
        self.__init__(rust_type)

    def to_dict(self) -> dict:
        ret = {
            "type_name": self.type_name,
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

    def get_description(self):
        return {
            "type": "InputSignal",
            "id": "_clock"
        }


@final
class _ScopeContext:
    def __init__(self, scope_level: SignalBase, epoch: SignalBase):
        self._scope = scope_level
        self._epoch = epoch

    def scoped(self, data: SignalBase, clock: SignalBase, default=None) -> SignalBase:
        from ..processors import EdgeTriggeredLatch, SignalMapper
        scope_starts = EdgeTriggeredLatch(control=self._scope, data=self._epoch)
        event_starts = EdgeTriggeredLatch(control=clock, data=self._epoch)
        return SignalMapper(
            bind_var="(sep, eep, signal)",
            lambda_src=f"""if *sep <= *eep {{ signal.clone() }} else {{
                {"Default::default()" if default is None else str(default)}
            }}""",
            upstream=[scope_starts, event_starts, data]
        ).annotate_type(data.get_rust_type_name())


class SessionizedInputSchemaBase(InputSchemaBase, ABC):
    def __init__(self, rust_type: RustCode = INPUT_SIGNAL_BAG):
        super().__init__(rust_type)
        self.session_signal = self.create_session_signal()
        self.epoch_signal = self.create_epoch_signal()
        self._sessionized_signals = dict()
        self._scope_ctx = _ScopeContext(scope_level=self.session_signal, epoch=self.epoch_signal)

    @abstractmethod
    def create_session_signal(self) -> SignalBase:
        raise NotImplementedError()

    @abstractmethod
    def create_epoch_signal(self) -> SignalBase:
        raise NotImplementedError()

    def _make_sessionized_input(self, key) -> SignalBase:
        if key not in self._sessionized_signals:
            raw_signal = super().__getattribute__(key)
            raw_signal_clock = raw_signal.clock()
            default_value = getattr(self, key + "_default", None)
            self._sessionized_signals[key] = self.sessionized(raw_signal,
                                                              raw_signal_clock,
                                                              default_value)
        return self._sessionized_signals[key]

    def sessionized(self, signal, signal_clock=None, default_value=None):
        if signal_clock is None:
            signal_clock = signal.clock()
        return self._scope_ctx.scoped(data=signal, clock=signal_clock, default=default_value)

    def __getattr__(self, name: str) -> Optional[SignalBase]:
        sessionized_prefix = "sessionized_"
        if name.startswith(sessionized_prefix):
            actual_key = name[len(sessionized_prefix):]
            return self._make_sessionized_input(actual_key)


def named(name: str, inner: _TypeBase = String()) -> MappedInputMember:
    return MappedInputMember(name, inner)


def volatile(inner: _TypeBase, default_value: RustCode = RUST_DEFAULT_VALUE) -> _TypeBase:
    inner._reset_expr = default_value
    return inner


def get_schema():
    return _defined_schema


def create_type_model_from_rust_type_name(rust_type: RustCode) -> Optional[_TypeBase]:
    if rust_type == "String":
        return String()
    elif rust_type[0] in ['i', 'u']:
        width = int(rust_type[1:])
        signed = rust_type[0] == 'i'
        return Integer(signed, width)
    elif rust_type[0] == 'f':
        width = int(rust_type[1:])
        return Float(width)
    elif rust_type[0] == 'bool':
        return Bool()
    else:
        return None