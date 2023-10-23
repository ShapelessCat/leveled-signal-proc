from abc import ABC
from json import dumps as dump_json_str
from typing import Optional

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

        Note: to register the type, the leveled signal should have a known type, otherwise, it's an error.
        """
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

    def has_been_true(self, duration = -1) -> SignalBase:
        """Shortcut for `has_been_true` module.

        Checks if the boolean signal has ever becomes true, and the result is a leveled signal.
        When `duration` is given, it checks if the signal has been true within `duration` amount of time.
        """
        from .modules import has_been_true
        return has_been_true(self, duration)

    def has_changed(self, duration = -1) -> SignalBase:
        """Shortcut for `has_changed` module.

        Checks if the signal has ever changed, and the result is a leveled signal.
        When `duration` is given, it checks if the signal has changed within `duration` amount of time.
        """
        from .modules import has_changed
        return has_changed(self, duration)

    def prior_different_value(self, scope: Optional[SignalBase] = None) -> SignalBase:
        return self.prior_value(self, scope)

    def prior_value(self, clock: Optional[SignalBase] = None, scope: Optional[SignalBase] = None) -> SignalBase:
        from .signal_processors import StateMachineBuilder
        if clock is None:
            clock = self.clock()
        ty = self.get_rust_type_name()
        builder = StateMachineBuilder(data = self, clock = clock)\
            .transition_fn(f'|(_, current): &({ty}, {ty}), data : &{ty}| (current.clone(), data.clone())')
        if scope is not None:
            builder.scoped(scope)
        return builder.build().annotate_type(f"({ty}, {ty})").map(
            bind_var = '(ret, _)',
            lambda_src = 'ret.clone()'
        ).annotate_type(self.get_rust_type_name())

    def measure_duration_true(self, scope_signal: Optional[SignalBase] = None) -> 'BuiltinMeasurementComponentBase':
        """Measures the total duration whenever this boolean signal is true.

        It returns a measurement.
        When `scope_signal` is given, it resets the duration to 0 when the `scope_signal` becomes a different level.
        """
        from .measurements import DurationTrue
        return DurationTrue(self, scope_signal = scope_signal)

    def measure_duration_since_true(self) -> 'BuiltinMeasurementComponentBase':
        """Measures the duration when this boolean signal has been true most recently.

        When the boolean signal is false, the output of the measurement is constantly 0.
        """
        from .measurements import DurationSinceBecomeTrue
        return DurationSinceBecomeTrue(self)

    def peek(self) -> 'BuiltinMeasurementComponentBase':
        """Returns the measurement that peek the latest value for the given signal.
        """
        from .measurements import Peek
        return Peek(self)


class BuiltinMeasurementComponentBase(BuiltinComponentBase, ABC):
    def __init__(self, name, **kwargs):
        super().__init__(**kwargs)
        self._namespace = f"lsp_component::measurements::{name}"


def get_components() -> list[LspComponentBase]:
    return _components


def serialize_defined_components(pretty_print=False) -> str:
    obj = [c.to_dict() for c in get_components()]
    return dump_json_str(obj, indent=4 if pretty_print else None)
