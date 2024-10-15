from abc import ABC

from ..rust_code import COMPILER_INFERABLE_TYPE, NAMESPACE_OP, RustCode
from .core import LeveledSignalProcessingModelComponentBase, MeasurementBase, SignalBase
from .schema import create_type_model_from_rust_type_name


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
    def __init__(
        self, package: str, namespace: RustCode, node_decl: RustCode, upstreams: list
    ):
        super().__init__(COMPILER_INFERABLE_TYPE)
        self._package = package
        self._namespace = namespace
        self._node_decl = node_decl
        self._upstreams = upstreams
        self._id = _assign_fresh_component_id()
        _components.append(self)

    def get_description(self):
        return {
            "type": "Component",
            "id": self._id,
        }

    def to_dict(self) -> dict[str, object]:
        upstreams = []
        for p in self._upstreams:
            if isinstance(p, list):
                upstreams.append(
                    {"type": "Tuple", "values": [e.get_description() for e in p]}
                )
            else:
                upstreams.append(p.get_description())
        return {
            "id": self._id,
            "is_measurement": not isinstance(self, SignalBase),
            "node_decl": self._node_decl,
            "upstreams": upstreams,
            "package": self._package,
            "namespace": self._namespace,
            "debug_info": self.debug_info,
        }


class BuiltinComponentBase(LspComponentBase, ABC):
    def __init__(self, component_package: RustCode, component_name: RustCode, **kwargs):
        package = "lsp-component"
        namespace = NAMESPACE_OP.join(
            [package.replace("-", "_"), component_package, component_name]
        )
        super().__init__(package, namespace, **kwargs)


class BuiltinProcessorComponentBase(BuiltinComponentBase, SignalBase, ABC):
    def __init__(self, name: RustCode, **kwargs):
        super().__init__(component_package="processors", component_name=name, **kwargs)


class BuiltinMeasurementComponentBase(BuiltinComponentBase, MeasurementBase, ABC):
    def __init__(self, name: RustCode, component_package: RustCode, **kwargs):
        super().__init__(component_package, component_name=name, **kwargs)

    def to_dict(self) -> dict[str, object]:
        return BuiltinComponentBase.to_dict(self)


class DirectBuiltinMeasurementComponentBase(BuiltinMeasurementComponentBase):
    def __init__(self, name: RustCode, **kwargs):
        super().__init__(name, component_package="measurements", **kwargs)


class IndirectBuiltinMeasurementComponentBase(BuiltinMeasurementComponentBase):
    # This is for codegen
    REFERENCE_PREFIX = "$"

    def __init__(
        self, name: RustCode, upstreams: list[SignalBase | MeasurementBase], **kwargs
    ):
        super().__init__(
            name,
            component_package=NAMESPACE_OP.join(["measurements", "combinator"]),
            upstreams=upstreams,
            **kwargs,
        )

    @staticmethod
    def get_id_or_literal_value(
        component: LeveledSignalProcessingModelComponentBase,
    ) -> str:
        from ..processors import Const

        if isinstance(component, Const):
            return component.rust_constant_value
        else:
            return IndirectBuiltinMeasurementComponentBase.REFERENCE_PREFIX + str(
                component.get_description()["id"]
            )


def get_components() -> list[LspComponentBase]:
    return _components
