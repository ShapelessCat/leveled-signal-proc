from json import dumps as dump_json_str
from lsdl.signal import LeveledSignalBase

def _make_assign_fresh_component_colsure():
    next_fresh_component_id = 0
    def assign_fresh_component() -> int:
        nonlocal next_fresh_component_id
        ret = next_fresh_component_id
        next_fresh_component_id += 1
        return ret
    return assign_fresh_component
    
_assign_fresh_component_id = _make_assign_fresh_component_colsure()
_components = []

class LspComponentBase(LeveledSignalBase):
    def __init__(self, is_measurement : bool, node_decl: str, upstreams: list):
        super().__init__()
        self._is_measurement = is_measurement
        self._node_decl = node_decl
        self._upstreams = upstreams
        self._namespace = ""
        self._package = ""
        self._id = _assign_fresh_component_id()
        self._output_type = "_"
        _components.append(self)
    def get_rust_type_name(self) -> str:
        return self._output_type
    def get_id(self):
        return {
            "type": "Component",
            "id": self._id,
        }
    def is_measurement(self) -> bool:
        return self._is_measurement
    def is_signal(self) -> bool :
        return not self.is_measurement()
    def to_dict(self) -> dict[str, object]:
        upstreams = []
        for p in self._upstreams:
            if type(p) == list:
                upstreams.append({
                    "type": "Tuple",
                    "values": [e.get_id() for e in p]
                })
            else:
                upstreams.append(p.get_id())
        return {
            "id": self._id,
            "is_measurement": self.is_measurement(),
            "node_decl": self._node_decl,
            "upstreams": upstreams,
            "package": self._package,
            "namespace": self._namespace,
            "debug_info": self._debug_info.to_dict(),
        }
    def add_metric(self, key):
        from lsdl import measurement_config
        if self.is_signal():
            measurement_config().add_metric(key, PeekValue(self))
        else:
            measurement_config().add_metric(key, self)

class SignalProcessor(LspComponentBase):
    def __init__(self, node_decl: str, upstreams: list):
        super().__init__(False, node_decl, upstreams)

class Measurement(LspComponentBase):
    def __init__(self, node_decl: str, upstreams: list):
        super().__init__(True, node_decl, upstreams)


def get_components() -> list[LspComponentBase]:
    return _components

class BuiltinComponentBase(LspComponentBase):
    def __init__(self, name, **kwargs):
        super().__init__(**kwargs)
        self._namespace = "lsp_component::{kind}::{name}".format(
            kind = "measurements" if self.is_measurement() else "processors",
            name = name
        )
        self._package = "lsp-component"

class SignalMapper(BuiltinComponentBase):
    def __init__(self, bind_var: str, lambda_src: str, upstream):
        lambda_decl = "|{bind_var}:&{bind_type}| {lambda_src}".format(
            bind_var = bind_var, 
            bind_type = upstream.get_rust_type_name() if type(upstream) != list else "(" + ",".join([e.get_rust_type_name() for e in upstream]) + ")",
            lambda_src = lambda_src,
        )
        node_decl = "SignalMapper::new({lambda_decl})".format(
            lambda_decl = lambda_decl
        )
        super().__init__(
            name = "SignalMapper",
            is_measurement = False, 
            node_decl = node_decl, 
            upstreams = [upstream]
        )

class Latch(BuiltinComponentBase):
    def __init__(self, control: LeveledSignalBase, data: LeveledSignalBase, forget_duration = -1):
        if forget_duration < 0:
            node_decl = "Latch::<{type_name}>::default()".format(type_name = data.get_rust_type_name())
        else:
            node_decl = "Latch::with_forget_behavior(<{type_name} as Default>::default(), <{type_name} as Default>::default(), {forget_duration})".format(
                type_name = data.get_rust_type_name(),
                forget_duration = forget_duration
            )
        super().__init__(
            name = "Latch",
            is_measurement = False, 
            node_decl = node_decl, 
            upstreams = [control, data]
        )

class ValueChangeCounter(BuiltinComponentBase):
    def __init__(self, input: LeveledSignalBase, init_val = 0):
        super().__init__(
            name = "ValueChangeCounter",
            is_measurement = False,
            node_decl = "ValueChangeCounter::with_init_value({val})".format(val = init_val),
            upstreams = [input]
        )

class Accumulator(BuiltinComponentBase):
    def __init__(self, control: LeveledSignalBase, data: LeveledSignalBase, init_val = None, filter_lambda = None):
        if filter_lambda is None:
            filter_lambda = "|_| true"
        if init_val is None:
            init_val = "Default::default()"
        node_decl = "Accumulator::<{dt},{ct}, _>::with_event_filter({init_val}, {filter_lambda})".format(
            dt = data.get_rust_type_name(),
            ct = control.get_rust_type_name(),
            init_val = init_val,
            filter_lambda = filter_lambda
        )
        super().__init__(
            name = "Accumulator",
            is_measurement = False,
            node_decl = node_decl,
            upstreams = [control, data]
        )

class DurationTrue(BuiltinComponentBase):
    def __init__(self, input: LeveledSignalBase):
        super().__init__(
            name = "DurationTrue",
            is_measurement = True,
            node_decl = "DurationTrue::default()",
            upstreams = [input]
        )

class PeekValue(BuiltinComponentBase):
    def __init__(self, input: LeveledSignalBase):
        super().__init__(
            name = "Peek",
            is_measurement = True,
            node_decl = "DurationTrue::default()",
            upstreams = [input]
        )


def get_defined_components() -> list[LspComponentBase]:
    return _components

def serialize_defined_components(pretty_print = False) -> list[dict[str, object]]:
    obj = [c.to_dict() for c in get_defined_components()]
    if pretty_print:
        return dump_json_str(obj, indent=4)
    else:
        return dump_json_str(obj)
         
         
