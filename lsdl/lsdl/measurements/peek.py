from typing import Tuple, final

from ..lsp_model.component_base import DirectBuiltinMeasurementComponentBase
from ..lsp_model.core import SignalBase
from ..rust_code import RustCode, RustPrimitiveType


@final
class Peek(DirectBuiltinMeasurementComponentBase):
    def __init__(self, input_signal: SignalBase):
        rule_component_name = self.__class__.__name__
        super().__init__(
            name=rule_component_name,
            node_decl=f"{rule_component_name}::default()",
            upstreams=[input_signal],
        )
        self.annotate_type(input_signal.get_rust_type_name())


@final
class PeekTimestamp(DirectBuiltinMeasurementComponentBase):
    def __init__(self, input_signal: SignalBase):
        rust_component_name = self.__class__.__name__
        super().__init__(
            name=rust_component_name,
            node_decl=f"{rust_component_name}",
            upstreams=[input_signal],
        )
        self.annotate_type(RustPrimitiveType.U64.value)

    BUILTIN_DATETIME_FORMATTER: Tuple[RustCode, RustCode] = (
        "nano_seconds",
        """{
            use std::time::{UNIX_EPOCH, Duration};
            use chrono::prelude::{DateTime, Utc};

            let d = UNIX_EPOCH + Duration::from_nanos(*nano_seconds);
            let datetime = DateTime::<Utc>::from(d);
            datetime.format("%Y-%m-%d %H:%M:%S.%3f UTC").to_string()
        }""",
    )
