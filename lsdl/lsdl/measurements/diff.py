from ..componet_base import DirectBuiltinMeasurementComponentBase
from ..signal import SignalBase


# This is acutally a subtype of the IndirectBuiltinMeasurementComponentBase
# TODO: Fix this later when adding direct measurement combinator support is done.
class DiffSinceCurrentLevel(DirectBuiltinMeasurementComponentBase):
    def __init__(self, control: SignalBase, data: SignalBase):
        rule_component_name = self.__class__.__name__
        super().__init__(
            name=rule_component_name,
            node_decl=f"{rule_component_name}::default()",
            upstreams=[control, data],
        )
        self.annotate_type(data.get_rust_type_name())
