import json
from abc import ABC, abstractmethod
from enum import StrEnum
from typing import Optional, Type, final, override

from ..rust_code import INPUT_SIGNAL_BAG, RUST_DEFAULT_VALUE, RustCode
from .core import LeveledSignalProcessingModelComponentBase, SignalBase


class SignalDataTypeBase(LeveledSignalProcessingModelComponentBase, ABC):
    def __init__(self, rust_type: RustCode):
        super().__init__(rust_type)
        self._reset_expr: Optional[RustCode] = None
        self._schema_entry: Optional[LeveledSignalProcessingModelComponentBase] = None

    @property
    def reset_expr(self):
        return self._reset_expr

    def get_description(self):
        if self._schema_entry is not None:
            return self._schema_entry.get_description()
        raise RuntimeError("This type `SignalDataTypeBase` should always have a corresponding schema entry")


class TypeWithLiteralValue(SignalDataTypeBase, ABC):
    @abstractmethod
    def render_rust_const(self, val, need_owned: bool = True) -> RustCode:
        raise NotImplementedError()


@final
class DateTime(SignalDataTypeBase):
    def __init__(self, timezone: RustCode = "Utc"):
        super().__init__(f"chrono::DateTime<chrono::{timezone}>")


@final
class String(TypeWithLiteralValue):
    def __init__(self):
        super().__init__("String")

    @override
    def render_rust_const(self, val, need_owned: bool = True) -> RustCode:
        s = json.dumps(val)
        return f"{s}.to_string()" if need_owned else s


@final
class Bool(TypeWithLiteralValue):
    def __init__(self):
        super().__init__("bool")

    @override
    def render_rust_const(self, val, _need_owned: bool = True) -> RustCode:
        return "true" if val else "false"


@final
class Integer(TypeWithLiteralValue):
    def __init__(self, signed=True, width=32):
        type_prefix = "i" if signed else "u"
        super().__init__(f"{type_prefix}{width}")

    @override
    def render_rust_const(self, val, _need_owned: bool = True) -> RustCode:
        return str(val) + self.get_rust_type_name()


@final
class Float(TypeWithLiteralValue):
    def __init__(self, width=64):
        super().__init__(f"f{width}")

    @override
    def render_rust_const(self, val, _need_owned: bool = True) -> RustCode:
        return str(val) + self.get_rust_type_name()


class LspEnumBase(StrEnum):
    def __str__(self) -> str:
        # Assume the `self.__class__.__name__` is upper camel case.
        # This should be true if we follow Python naming conventions.
        return f"{self.__class__.__name__}::{LspEnumBase.upper_camel_case(self.name)}"

    @classmethod
    def variants_info(cls) -> list[dict[str, str]]:
        """Generate IR for the variants of current enum"""
        return [
            {
                "variant_name": LspEnumBase.upper_camel_case(v.name),
                "input_value": v.value,
            }
            for v in cls
        ]

    @staticmethod
    def upper_camel_case(python_identifier: str) -> str:
        return "".join(seg.lower().capitalize() for seg in python_identifier.split("_"))


@final
class CStyleEnum(TypeWithLiteralValue):
    def __init__(self, python_str_enum: Type[LspEnumBase]):
        super().__init__(python_str_enum.__name__)
        self.str_enum_type = python_str_enum

    @override
    def render_rust_const(self, val: RustCode, _need_owned: bool = True) -> RustCode:
        maybe_variant = self.str_enum_type(val)
        if maybe_variant in self.str_enum_type:
            return str(maybe_variant)
        else:
            raise ValueError(
                f"Value {val} should be a variant of {self.get_rust_type_name()}: [{", ".join(v.value for v in self.str_enum_type)}]"
            )


@final
class Vector(TypeWithLiteralValue):
    def __init__(self, element_type: SignalDataTypeBase):
        super().__init__(f"Vec<{element_type.get_rust_type_name()}>")
        self._element_type = element_type

    @override
    def render_rust_const(self, val, _need_owned: bool = True) -> RustCode:
        if isinstance(self._element_type, TypeWithLiteralValue):
            typed_const_elements = ", ".join(
                [self._element_type.render_rust_const(v) for v in val]
            )
            return f"vec![{typed_const_elements}]"
        else:
            raise Exception("Not a vector literal!")


class _InputMember(SignalBase, ABC):
    def __init__(self, tpe: SignalDataTypeBase, name=""):
        super().__init__(tpe.get_rust_type_name())
        tpe._schema_entry = self
        self._signal_data_type = tpe
        self._name = name
        self._reset_expr: Optional[RustCode] = None

    @property
    def signal_data_type(self) -> SignalDataTypeBase:
        return self._signal_data_type

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
    def __init__(
        self,
        input_key: str,
        tpe: SignalDataTypeBase,
        volatile_default_value: Optional[RustCode] = None,
    ):
        super().__init__(tpe)
        self._input_key = input_key
        self._reset_expr = volatile_default_value or self.signal_data_type.reset_expr

    @property
    def reset_expr(self):
        return self._reset_expr

    def get_input_key(self) -> str:
        return self._input_key

    def clock(self) -> _ClockCompanion:
        # This `self.name` will be given when initializing the `InputSchemaBase` through reflection.
        return _ClockCompanion(f"{self.name}_clock")

    # TODO: move this outside of this class or convert them to static methods!!!
    def parse(
        self, type_name, default_value: RustCode = RUST_DEFAULT_VALUE
    ) -> SignalBase:
        return self.map(
            bind_var="s",
            lambda_src=f"s.parse::<{type_name}>().unwrap_or({default_value})",
        ).annotate_type(type_name)

    def starts_with(self, other) -> SignalBase:
        from ..processors import Const, make_tuple

        if not isinstance(other, LeveledSignalProcessingModelComponentBase):
            other = Const(other)
        return (
            make_tuple(self, other)
            .map(bind_var="(s, p)", lambda_src="s.starts_with(p)")
            .annotate_type("bool")
        )

    def timestamp(self):
        return self.map(bind_var="t", lambda_src="t.timestamp()").annotate_type("i64")


_defined_schema: Optional["InputSchemaBase"] = None


class InputSchemaBase(SignalBase):
    def __init__(self, type_name: RustCode = INPUT_SIGNAL_BAG):
        global _defined_schema
        self.type_name = type_name
        # If treating `InputSchemaBase` itself as a signal/clock, its type should be `u64`.
        # Actually, lsp-codegen will always automatically insert a `_clock: u64` field
        # to the codegen result struct of this class, and the generated struct name should
        # be the value of `self.type_name`.
        super().__init__("u64")
        self._member_names = []
        if "_timestamp_key" not in self.__dir__():
            self._timestamp_key = "timestamp"
        for item_name in self.__dir__():
            item = self.__getattribute__(item_name)
            # There won't be members as `ClockCompanion`s in the source code of
            # an `InputSchemaBase` instance, therefore we don't try to handle it here.
            if isinstance(item, SignalDataTypeBase):
                item = MappedInputMember(input_key=item_name, tpe=item)
            if isinstance(item, MappedInputMember):
                item.name = item_name
                self.__setattr__(item_name, item)
                self._member_names.append(item_name)
        _defined_schema = self

    def to_dict(self) -> dict:
        ret: dict = {
            "type_name": self.type_name,
            "patch_timestamp_key": self._timestamp_key,
            "members": {},
        }
        for name in self._member_names:
            member: MappedInputMember = getattr(self, name)
            ret["members"][name] = {
                "type": member.get_rust_type_name(),
                "clock_companion": member.clock().name,
                "input_key": member.get_input_key(),
                "debug_info": member.debug_info.to_dict(),
            }
            if isinstance(enum := member.signal_data_type, CStyleEnum):
                ret["members"][name][
                    "enum_variants"
                ] = enum.str_enum_type.variants_info()
                # ret["members"][name]["enum_variants"] = enum.variants
            if member.reset_expr is not None:
                ret["members"][name]["signal_behavior"] = {
                    "name": "Reset",
                    "default_expr": member.reset_expr,
                }
        return ret

    def get_description(self):
        return {"type": "InputSignal", "id": "_clock"}


@final
class _ScopeContext:
    def __init__(self, scope_level: SignalBase, epoch: SignalBase):
        self._scope = scope_level
        self._epoch = epoch

    def scoped(
        self, data: SignalBase, clock: SignalBase, default: RustCode
    ) -> SignalBase:
        from ..processors import EdgeTriggeredLatch, SignalMapper

        scope_starts = EdgeTriggeredLatch(control=self._scope, data=self._epoch)
        event_starts = EdgeTriggeredLatch(control=clock, data=self._epoch)
        return SignalMapper(
            bind_var="(sep, eep, signal)",
            lambda_src=f"if *sep <= *eep {{ signal.clone() }} else {{ {default} }}",
            upstream=[scope_starts, event_starts, data],
        ).annotate_type(data.get_rust_type_name())


class SessionizedInputSchemaBase(InputSchemaBase, ABC):
    SESSIONIZED_PREFIX = "sessionized_"
    SESSIONIZED_PREFIX_SIZE = len(SESSIONIZED_PREFIX)

    def __init__(self, rust_type: RustCode = INPUT_SIGNAL_BAG):
        super().__init__(rust_type)
        self.session_signal: SignalBase = self.create_session_signal()
        self.epoch_signal: SignalBase = self.create_epoch_signal()
        self._sessionized_signals: dict[str, SignalBase] = dict()
        self._scope_ctx = _ScopeContext(
            scope_level=self.session_signal, epoch=self.epoch_signal
        )

    @abstractmethod
    def create_session_signal(self) -> SignalBase:
        raise NotImplementedError()

    @abstractmethod
    def create_epoch_signal(self) -> SignalBase:
        raise NotImplementedError()

    def _make_sessionized_input(self, input_member_name: str) -> SignalBase:
        if input_member_name not in self._sessionized_signals:
            raw_signal = super().__getattribute__(input_member_name)
            if not isinstance(raw_signal, MappedInputMember):
                raise TypeError(f"{input_member_name} must be from input signal bag")

            attr = f"{input_member_name}_default"
            # Don't use `hasattr` and `getattr`, both can potentially trigger
            # `__getattr__`.  I can modify the below `__getattr__`'s `else`
            # branch to make it return `None`, which can be a workaround to make
            # the code here much simpler, but the type hint of `__getattr__`
            # will become `Optional<SignalBase>` instead of `SignalBase`, which
            # is not what we expected.
            default_value: RustCode = (
                str(self.__getattribute__(attr))
                if attr in self.__dict__
                else RUST_DEFAULT_VALUE
            )
            self._sessionized_signals[input_member_name] = self._sessionized(
                raw_signal, default_value
            )
        return self._sessionized_signals[input_member_name]

    def _sessionized(
        self, signal: MappedInputMember, default_value: RustCode
    ) -> SignalBase:
        return self._scope_ctx.scoped(
            data=signal, clock=signal.clock(), default=default_value
        )

    def __getattr__(self, name: str) -> SignalBase:
        if name.startswith(self.SESSIONIZED_PREFIX):
            actual_key = name[self.SESSIONIZED_PREFIX_SIZE :]
            return self._make_sessionized_input(actual_key)
        else:
            raise ValueError(f"Signal {name} doesn't exist")


def named(
    name: str,
    inner: SignalDataTypeBase = String(),
    *,
    volatile_default_value: Optional[RustCode] = None,
) -> MappedInputMember:
    return MappedInputMember(name, inner, volatile_default_value)


def volatile(
    inner: SignalDataTypeBase, default_value: RustCode = RUST_DEFAULT_VALUE
) -> SignalDataTypeBase:
    inner._reset_expr = default_value
    return inner


def get_schema():
    return _defined_schema


def create_type_model_from_rust_type_name(
    rust_type: RustCode,
) -> Optional[SignalDataTypeBase]:
    if rust_type == "String":
        return String()
    elif rust_type[0] in ["i", "u"]:
        width = int(rust_type[1:])
        signed = rust_type[0] == "i"
        return Integer(signed, width)
    elif rust_type[0] == "f":
        width = int(rust_type[1:])
        return Float(width)
    elif rust_type[0] == "bool":
        return Bool()
    else:
        return None
