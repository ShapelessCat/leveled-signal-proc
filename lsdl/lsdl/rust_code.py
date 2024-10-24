from enum import StrEnum, auto
from typing import final


type RustCode = str

RUST_DEFAULT_VALUE: RustCode = "Default::default()"
INPUT_SIGNAL_BAG: RustCode = "InputSignalBag"

COMPILER_INFERABLE_TYPE: RustCode = "_"
NAMESPACE_OP: RustCode = "::"

@final
class RustPrimitiveType(StrEnum):
    # Floating-Point Types
    F32 = auto()
    F64 = auto()

    # The Boolean Type
    BOOL = auto()

    # The Char Type
    CHAR = auto()

    # The String Type
    STRING = "String"

    # Integer Types
    I8 = auto()
    U8 = auto()

    I16 = auto()
    U16 = auto()

    I32 = auto()
    U32 = auto()

    I64 = auto()
    U64 = auto()

    I128 = auto()
    U128 = auto()

    ISIZE = auto()
    USIZE = auto()