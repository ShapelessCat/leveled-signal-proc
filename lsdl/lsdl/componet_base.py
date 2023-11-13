import configparser
import os
from abc import ABC

from .lsp_model_component import LeveledSignalProcessingModelComponentBase
from .measurement import MeasurementBase
from .rust_code import COMPILER_INFERABLE_TYPE, RustCode
from .schema import create_type_model_from_rust_type_name
from .signal import SignalBase

__config = configparser.ConfigParser()
__current_file_path = os.path.dirname(os.path.abspath(__file__))
__config.read(f"{__current_file_path}/rust_keywords.ini")
_strict_and_reserved_rust_keywords = {*__config['strict'].values(), *__config['reserved'].values()}


def _make_assign_fresh_component_closure():
    next_fresh_component_id = 0

    def assign_fresh_component() -> int:
        nonlocal next_fresh_component_id
        ret = next_fresh_component_id
        next_fresh_component_id += 1
        return ret
    return assign_fresh_component


_assign_fresh_component_id = _make_assign_fresh_component_closure()
_components = []


class LspComponentBase(LeveledSignalProcessingModelComponentBase, ABC):
    def __init__(self, package: str, namespace: RustCode, node_decl: RustCode, upstreams: list):
        super().__init__()
        self._package = package
        self._namespace = namespace
        self._node_decl = node_decl
        self._upstreams = upstreams
        self._id = _assign_fresh_component_id()
        self._output_type = COMPILER_INFERABLE_TYPE
        _components.append(self)

    def __getattribute__(self, __name: str):
        try:
            return super().__getattribute__(__name)
        except AttributeError as e:
            type_model = create_type_model_from_rust_type_name(self._output_type)
            type_model._parent = self
            if type_model is not None:
                return getattr(type_model, __name)
            else:
                raise e

    def annotate_type(self, typename: RustCode):
        self._output_type = typename
        return self

    def get_rust_type_name(self) -> RustCode:
        return self._output_type

    def get_id(self):
        return {
            "type": "Component",
            "id": self._id,
        }

    def add_metric(self, key, typename: RustCode = COMPILER_INFERABLE_TYPE) -> 'LspComponentBase':
        """Register the leveled signal as a metric.

        The registered metric results will present in the output data structure.

        Note:
        to register the type, the leveled signal should have a known type, otherwise, it's an error.
        """
        LspComponentBase.validate_rust_identifier(key)
        from . import measurement_config
        from .measurements import Peek
        if isinstance(self, SignalBase):
            measurement_config().add_metric(key, Peek(self), typename)
        else:
            measurement_config().add_metric(key, self, typename)
        return self

    # TODO: We should allow all legal Rust identifiers.
    @classmethod
    def validate_rust_identifier(cls, identifier: str) -> None:
        """Check if an identifier is a legal Rust identifier.

        For implementation simplicity, only a C-style identifier that is not a Rust strict/reserved keyword is allowed.

        CAUTION:
        Current check is easy to implement, but it is also too strict. We should allow all legal Rust identifier.
        """
        import re
        regex = '^[A-Za-z_][A-Za-z0-9_]*$'
        if not re.match(regex, identifier) or identifier in _strict_and_reserved_rust_keywords:
            raise Exception(f"{identifier} is not a simple and legal Rust identifier!")

    def to_dict(self) -> dict[str, object]:
        upstreams = []
        for p in self._upstreams:
            if isinstance(p, list):
                upstreams.append({
                    "type": "Tuple",
                    "values": [e.get_id() for e in p]
                })
            else:
                upstreams.append(p.get_id())
        return {
            "id": self._id,
            "is_measurement": not isinstance(self, SignalBase),
            "node_decl": self._node_decl,
            "upstreams": upstreams,
            "package": self._package,
            "namespace": self._namespace,
            "debug_info": self.debug_info.to_dict(),
        }


class BuiltinComponentBase(LspComponentBase, ABC):
    def __init__(self, component_package: RustCode, component_name: RustCode, **kwargs):
        package = "lsp-component"
        namespace = f"{package.replace('-', '_')}::{component_package}::{component_name}"
        super().__init__(package, namespace, **kwargs)


class BuiltinProcessorComponentBase(BuiltinComponentBase, SignalBase, ABC):
    def __init__(self, name: RustCode, **kwargs):
        super().__init__(component_package="processors", component_name=name, **kwargs)


class BuiltinMeasurementComponentBase(BuiltinComponentBase, MeasurementBase, ABC):
    def __init__(self, name: RustCode, **kwargs):
        super().__init__(component_package="measurements", component_name=name, **kwargs)


def get_components() -> list[LspComponentBase]:
    return _components
