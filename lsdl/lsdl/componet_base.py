from abc import ABC
from json import dumps as dump_json_str

from .schema import create_type_model_from_rust_type_name
from .signal import LeveledSignalProcessingModelComponentBase, SignalBase


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
    def __init__(self, node_decl: str, upstreams: list):
        super().__init__()
        self._node_decl = node_decl
        self._upstreams = upstreams
        self._namespace = ""
        self._package = ""
        self._id = _assign_fresh_component_id()
        self._output_type = "_"
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

    def annotate_type(self, typename: str):
        self._output_type = typename
        return self

    def get_rust_type_name(self) -> str:
        return self._output_type

    def get_id(self):
        return {
            "type": "Component",
            "id": self._id,
        }

    def add_metric(self, key, typename = "_") -> 'LspComponentBase':
        """Register the leveled signal as a metric.

        The registered metric results will present in the output data structure.

        Note:
        to register the type, the leveled signal should have a known type, otherwise, it's an error.
        """
        # TODO: Check if the `key` is a valid Rust identifier!
        from . import measurement_config
        from .measurements import Peek
        if isinstance(self, SignalBase):
            measurement_config().add_metric(key, Peek(self), typename)
        else:
            measurement_config().add_metric(key, self, typename)
        return self

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
    def __init__(self, **kwargs):
        super().__init__(**kwargs)
        self._package = "lsp-component"
        self._rust_component_name = self.__class__.__name__


class BuiltinProcessorComponentBase(BuiltinComponentBase, SignalBase, ABC):
    def __init__(self, name, **kwargs):
        super().__init__(**kwargs)
        self._namespace = f"lsp_component::processors::{name}"


class BuiltinMeasurementComponentBase(BuiltinComponentBase, ABC):
    def __init__(self, name, **kwargs):
        super().__init__(**kwargs)
        self._namespace = f"lsp_component::measurements::{name}"


def get_components() -> list[LspComponentBase]:
    return _components


def serialize_defined_components(pretty_print=False) -> str:
    obj = [c.to_dict() for c in get_components()]
    return dump_json_str(obj, indent=4 if pretty_print else None)
