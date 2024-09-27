import json
from abc import ABC, abstractmethod
from typing import Optional, final

# from .core import LeveledSignalProcessingModelComponentBase, SignalBase
# from ..rust_code import INPUT_SIGNAL_BAG, RUST_DEFAULT_VALUE, RustCode

from lsdl.lsp_model.core import LeveledSignalProcessingModelComponentBase, SignalBase
from lsdl.lsp_model.schema import _TypeBase, MappedInputMember, named
from lsdl.rust_code import INPUT_SIGNAL_BAG, RUST_DEFAULT_VALUE, RustCode


@final
class Object(_TypeBase):
    def __init__(self, name: RustCode):
        super().__init__(rust_type=name)


_defined_schema: Optional["InputSchemaBase"] = None


class NestableInputSchemaBase(SignalBase):
    def __init__(self, type_name: RustCode = INPUT_SIGNAL_BAG):
        global _defined_schema
        self.type_name = type_name
        # If treating `InputSchemaBase` itself as a signal/clock, its type should be `u64`.
        # Actually, lsp-codegen will always automatically insert a `_clock: u64` field
        # to the codegen result struct of this class, and the generated struct name should
        # be the value of `self.type_name`.
        super().__init__("u64")
        self._members = []
        if "_timestamp_key" not in self.__dir__():
            self._timestamp_key = "timestamp"

        self.generate_inner_members(self)
        # #vvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvv
        # #---------------------------------------
        # # Need refactor
        # #---------------------------------------
        # for item_name in self.__dir__():
        #     item = self.__getattribute__(item_name)

        #     # print(item.__dir__)
        #     if isinstance(item, Struct):
        #         for inner in item.__dir__():
        #             inner_item = item.__getattribute__(inner)
        #             if isinstance(inner_item, _TypeBase):
        #                 print(f"Inner: {inner}")
        #                 # x_item = self.__getattribute__(x)
        #                 # if isinstance(item, _TypeBase):
        #                 #     print(f"Inner: {x}")

        #     # #vvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvv
        #     # print(f"-- {item} --")
        #     # if not item_name.startswith("__") and isinstance(item, _TypeBase):
        #     #     print(item_name)
        #     # #^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
        #     # There won't be members as `ClockCompanion`s in the source code of
        #     # an `InputSchemaBase` instance, therefore we don't try to handle it here.
        #     if isinstance(item, _TypeBase):
        #         item = MappedInputMember(input_key=item_name, tpe=item)

        #     if isinstance(item, MappedInputMember):
        #         item.name = item_name
        #         self.__setattr__(item_name, item)
        #         self._members.append(item_name)

        _defined_schema = self
    
    def generate_inner_members(self, container_item):
        #vvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvv
        #---------------------------------------
        # Need refactor
        #---------------------------------------
        for item_name in container_item.__dir__():
            item = container_item.__getattribute__(item_name)

            if isinstance(item, MappedInputMember):
                print(item_name)
                self._members.append(item_name)
                if item.isinstance(Object):
                    self.generate_inner_members(item)


    def to_dict(self) -> dict:
        ret: dict = {
            "type_name": self.type_name,
            "patch_timestamp_key": self._timestamp_key,
            "members": {},
        }
        for member in self._members:
            match getattr(self, member, None):
                case None:
                    pass
                case member_type:
                    ret["members"][member] = {
                        "type": member_type.get_rust_type_name(),
                        "clock_companion": member_type.clock().name,
                        "input_key": member_type.get_input_key(),
                        "debug_info": member_type.debug_info.to_dict(),
                    }
                    if member_type.reset_expr is not None:
                        ret["members"][member]["signal_behavior"] = {
                            "name": "Reset",
                            "default_expr": member_type.reset_expr,
                        }
                
            member_type = getattr(self, member)
            ret["members"][member] = {
                "type": member_type.get_rust_type_name(),
                "clock_companion": member_type.clock().name,
                "input_key": member_type.get_input_key(),
                "debug_info": member_type.debug_info.to_dict(),
            }
            if member_type.reset_expr is not None:
                ret["members"][member]["signal_behavior"] = {
                    "name": "Reset",
                    "default_expr": member_type.reset_expr,
                }
        return ret

    def get_description(self):
        return {"type": "InputSignal", "id": "_clock"}


from lsdl.lsp_model import InputSchemaBase, String, Float


class NestableInputSignal(NestableInputSchemaBase):
    _timestamp_key = "timestamp"

    fundamental = named("fundamental", Object("fundamental"))

    fundamental.event_name = named("event_name")
    fundamental.event_category = named("event_category")

    fundamental.inner = named("inner", Object("inner"))
    fundamental.inner.test = named("test", Float())

    encoded_fps = named("encoded_fps", Float())  # noqa: E221
    inferred_rendered_fps = named("inferred_rendered_fps", Float())  # noqa: E221


if __name__ == "__main__":
    nestable_input_signal = NestableInputSignal()

    repr = nestable_input_signal.to_dict()
    print(json.dumps(repr, indent=4), end='\n\n')

    # desc = nestable_input_signal.get_description()
    # print(json.dumps(desc, indent=4))