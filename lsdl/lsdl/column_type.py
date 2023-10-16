
import json
from typing import override
from abc import ABC, abstractmethod

from .signal import LeveledSignalProcessingModelComponentBase


class TypeBase(LeveledSignalProcessingModelComponentBase, ABC):
    def __init__(self, rs_type: str):
        super().__init__()
        self._rust_type = rs_type
        self._reset_expr = None
        self._parent = None

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

    def timestamp(self):
        return self\
            .map(
                bind_var = "t",
                lambda_src = "t.timestamp()"
            ).annotate_type("i64")


class String(TypeWithLiteralValue):
    def __init__(self):
        super().__init__("String")

    @override
    def render_rust_const(self, val) -> str:
        return f"{json.dumps(val)}.to_string()"

    def parse(self, type_name, default_value = "Default::default()") -> LeveledSignalProcessingModelComponentBase:
        return self\
            .map(
                bind_var = "s",
                lambda_src = f"s.parse::<{type_name}>().unwrap_or({default_value})"
            ).annotate_type(type_name)

    def starts_with(self, other) -> LeveledSignalProcessingModelComponentBase:
        from .const import Const
        from .modules import make_tuple
        if not isinstance(other, LeveledSignalProcessingModelComponentBase):
            other = Const(other)
        return make_tuple(self, other)\
            .map(
                bind_var = "(s, p)",
                lambda_src = "s.starts_with(p)"
            ).annotate_type("bool")


class Bool(TypeWithLiteralValue):
    def __init__(self):
        super().__init__("bool")

    @override
    def render_rust_const(self, val) -> str:
        return "true" if val else "false"


class Integer(TypeWithLiteralValue):
    def __init__(self, signed = True, width = 32):
        super().__init__("i" + str(width) if signed else "u" + str(width))

    @override
    def render_rust_const(self, val) -> str:
        return str(val) + self.get_rust_type_name()


class Float(TypeWithLiteralValue):
    def __init__(self, width = 64):
        super().__init__("f" + str(width))

    @override
    def render_rust_const(self, val) -> str:
        return str(val) + self.get_rust_type_name()


class Vector(TypeWithLiteralValue):
    def __init__(self, inner: TypeBase):
        super().__init__("Vec<" + inner.get_rust_type_name() + ">")
        self._inner = inner

    @override
    def render_rust_const(self, val) -> str:
        typed_const_elements = ",".join([self._inner.render_rust_const(v) for v in val])
        return f"vec![{typed_const_elements}]"