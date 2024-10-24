
import json
from typing import Any, Optional, final

from lsdl import RustCode
from lsdl.lsp_model.schema import (
    _InputMember,
    Bool,
    CStyleEnum,
    DateTime,
    Float,
    Integer,
    LspEnumBase,
    MappedInputMember,
    Object,
    SignalDataTypeBase,
    String,
    Vector,
    named,
)
from lsdl.rust_code import INPUT_SIGNAL_BAG

_defined_nested_schema: Optional["GeneralInputSchemaBase"] = None


class GeneralInputSchemaBase(_InputMember):
    PROPERTIES_KEY = "properties"
    DEFS = "$defs"

    def __init__(self, type_name: RustCode = INPUT_SIGNAL_BAG):
        global _defined_nested_schema
        self.type_name = type_name
        # If treating `InputSchemaBase` itself as a signal/clock, its type should be `u64`.
        # Actually, lsp-codegen will always automatically insert a `_clock: u64` field
        # to the codegen result struct of this class, and the generated struct name should
        # be the value of `self.type_name`.
        super().__init__(Integer(signed=False, width=64), type_name)
        if "_timestamp_key" not in vars(self.__class__):
            self._timestamp_key = "timestamp"
        self._top_level_member_names = []
        for item_name, item in vars(self.__class__).items():
            # There won't be members as `ClockCompanion`s in the source code of
            # an `InputSchemaBase` instance, therefore we don't try to handle it
            # here.
            if isinstance(item, SignalDataTypeBase | MappedInputMember):
                _normalize(self, item_name, item)
                self._top_level_member_names.append(item_name)
        _defined_nested_schema = self
    
    def to_schema(self) -> dict:
        whole_signal_clock_name = "_clock"
        ret: dict = {
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "$comment": "InputSignalBag schema",
            "title": self.type_name,
            "type": "object",
            GeneralInputSchemaBase.PROPERTIES_KEY: {
                whole_signal_clock_name: {
                    "$ref": f"#/$def/{self.signal_data_type.get_rust_type_name()}"
                },
            },
            "required": [whole_signal_clock_name],
            "$defs": {
                "u64": {  ## For input signal clock
                    "type": "integer",
                    "x-rust-type": {
                        "crate": "std",
                        "version": "*",
                        "path": f"std::primitive::u64"
                    },
                    "debug_info": self.debug_info,
                },
            },
        }

        properties = ret[GeneralInputSchemaBase.PROPERTIES_KEY]

        for name in self._top_level_member_names:
            member: MappedInputMember = getattr(self, name)
            match member.signal_data_type:
                case Object() | DateTime():
                    properties[member.name] = { "$ref": f"#/$defs/{member.name}" }
                case _:
                    properties[member.name] = { "$ref": f"#/$defs/{member.signal_data_type.get_rust_type_name()}" }
            ret["required"].append(member.name)

            if not isinstance(member.signal_data_type, Object):
                clock = member.clock()
                def_name = clock.signal_data_type.get_rust_type_name()
                properties[clock.name] =  { "$ref": f"#/$defs/{def_name}" }
                ret["required"].append(clock.name)
            GeneralInputSchemaBase._process_for_schema(member, name, ret)
        return ret

    @staticmethod
    def _process_for_schema(member: MappedInputMember, name: str, ret: dict[str, Any]):
        defs = ret[GeneralInputSchemaBase.DEFS]

        match member.signal_data_type:
            case Object() | CStyleEnum() | DateTime():
                defs[name] = member.signal_data_type.schema_ir | {
                    "debug_info": member.debug_info,
                }
                if isinstance(enum := member.signal_data_type, CStyleEnum):
                    defs[name][
                        "enum_variants"
                    ] = enum.str_enum_type.variants_info()
                    defs[name][
                        "enum"
                    ] = [pair["input_value"] for pair in enum.str_enum_type.variants_info()]
            case _:
                defs[member.signal_data_type.get_rust_type_name()] = member.signal_data_type.schema_ir | {
                    "debug_info": member.debug_info,
                }

        # TODO: RANK0
        # if member.reset_expr is not None:
        #     defs[name]["signal_behavior"] = {
        #         "name": "Reset",
        #         "default_expr": member.reset_expr,
        #     }

        if not isinstance(member.signal_data_type, Object):
            clock = member.clock()
            def_name = clock.signal_data_type.get_rust_type_name()
            defs[def_name] = clock.signal_data_type.schema_ir | {
                "debug_info": member.debug_info,
            }
        else:
            defs[name]["required"] = []
            required: list[str] = defs[name]["required"]

            defs[name][GeneralInputSchemaBase.PROPERTIES_KEY] = {}
            inner_property = defs[name][GeneralInputSchemaBase.PROPERTIES_KEY]

            for inner_name in member.__dict__:
                inner_member = member.__getattribute__(inner_name)
                if isinstance(inner_member, MappedInputMember):
                    match inner_member.signal_data_type:
                        case Object() | DateTime():
                            inner_property[inner_name] = { "$ref": f"#/$defs/{inner_name}" }
                        case _:
                            inner_property[inner_name] = { "$ref": f"#/$defs/{inner_member.signal_data_type.get_rust_type_name()}" }
                    required.append(inner_name)

                    # inner_property[inner_name] = { "$ref": f"#/$defs/{inner_name}" }
                    # required.append(inner_name)

                    clock = inner_member.clock()
                    clock_def_name = inner_member.clock().signal_data_type.get_rust_type_name()
                    if not isinstance(inner_member.signal_data_type, Object):
                        inner_property[clock.name] = { "$ref": f"#/$defs/{clock_def_name}" }
                        required.append(clock.name)

                    # clock_name = inner_member.clock().name
                    # if not isinstance(inner_member.signal_data_type, Object):
                    #     inner_property[clock_name] = { "$ref": f"#/$defs/{clock_name}" }
                    #     required.append(clock_name)

                    GeneralInputSchemaBase._process_for_schema(inner_member, inner_name, ret)

    def to_patch_schema(self) -> dict:
        return self.to_dict()

    def to_dict(self) -> dict:
        ret: dict = {
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "$comment": "InputSignalBagPatch schema",
            "title": f"{self.type_name}Patch",
            "type": "object",
            # "patch_timestamp_key": self._timestamp_key,
            GeneralInputSchemaBase.PROPERTIES_KEY: {
                "timestamp": { "$ref": f"#/$defs/timestamp" }
            },
            "$defs": {
                "timestamp": {
                    "type": "string",
                    "format": "date-time"
                }
            },
            "required": ["timestamp"],  # TODO: timestamp
        }

        properties = ret[GeneralInputSchemaBase.PROPERTIES_KEY]

        for name in self._top_level_member_names:
            member: MappedInputMember = getattr(self, name)
            patch_name = f"{name}_patch" if isinstance(member.signal_data_type, Object | DateTime) else member.get_rust_type_name()
            properties[name] = { "$ref": f"#/$defs/{patch_name}" }
            if isinstance(member.signal_data_type, Object):
                ret["required"].append(name)
            GeneralInputSchemaBase._process_for_patch_schema(member, patch_name, ret)
        return ret

    @staticmethod
    def _process_for_patch_schema(member: MappedInputMember, name: str, ret: dict[str, Any]):
        defs = ret[GeneralInputSchemaBase.DEFS]

        match member.signal_data_type:
            case Object() | DateTime() | CStyleEnum():
                defs[name] = member.signal_data_type.schema_ir | {
                    "input_key": member.input_key,
                    "debug_info": member.debug_info,
                }
                if isinstance(enum := member.signal_data_type, CStyleEnum):
                    defs[name][
                        "enum_variants"
                    ] = enum.str_enum_type.variants_info()
                    defs[name][
                        "enum"
                    ] = [pair["input_value"] for pair in enum.str_enum_type.variants_info()]
            case _:
                defs[member.get_rust_type_name()] = member.signal_data_type.schema_ir | {
                    "input_key": member.input_key,
                    "debug_info": member.debug_info,
                }

        if isinstance(member.signal_data_type, Object):
            defs[name]["required"] = []
            required: list[str] = defs[name]["required"]

            defs[name][GeneralInputSchemaBase.PROPERTIES_KEY] = {}
            inner_property = defs[name][GeneralInputSchemaBase.PROPERTIES_KEY]

            for inner_name in member.__dict__:
                inner_member = member.__getattribute__(inner_name)
                if isinstance(inner_member, MappedInputMember):
                    inner_patch_name = f"{inner_name}_patch" if isinstance(inner_member.signal_data_type, Object | DateTime) else inner_member.get_rust_type_name()
                    inner_property[inner_name] = { "$ref": f"#/$defs/{inner_patch_name}" }
                    if isinstance(inner_member.signal_data_type, Object):
                        required.append(inner_name)
                    GeneralInputSchemaBase._process_for_patch_schema(inner_member, inner_patch_name, ret)

    def get_description(self):
        return {"type": "InputSignal", "id": "_clock"}
    

def _normalize(parent: GeneralInputSchemaBase | MappedInputMember, input_key: str, node: SignalDataTypeBase | MappedInputMember) -> None:
    match node:
        case SignalDataTypeBase():
            item = MappedInputMember(input_key, tpe=node)
            item.name = input_key
            parent.__setattr__(input_key, item)
            existing_attributes = node.__class__.__dict__
            for child_item_name, child_item in existing_attributes.items():
                if isinstance(child_item, SignalDataTypeBase | MappedInputMember):
                    _normalize(parent=item, input_key=child_item_name, node=child_item)
        case MappedInputMember():
            node.name = input_key
            existing_attributes = node.__class__.__dict__
            for child_item_name, child_item in existing_attributes.items():
                if isinstance(child_item, SignalDataTypeBase | MappedInputMember):
                    _normalize(parent=node, input_key=child_item_name, node=child_item)


## TEST
@final
class Currency(LspEnumBase):
    Unknown = "UNKNOWN"
    Cny = "CNY"
    Euro = "EURO"
    Usd = "USD"


class Inner(Object):
    test = Float()


class Fundamental(Object):
    event_name = String()
    event_category = String()
    inner = Inner()


class NestableInputSignal(GeneralInputSchemaBase):
    _timestamp_key = "timestamp"

    fundamental = Fundamental()

    encoded_fps = Float()
    inferred_rendered_fps = Float()

    currency = CStyleEnum(Currency)

    i8_int = Integer(signed=True, width=8)
    u8_int = Integer(signed=False, width=8)

    i16_int = Integer(signed=True, width=16)
    u16_int = Integer(signed=False, width=16)

    i32_int = Integer()
    u32_int = Integer(signed=False, width=32)

    i64_int = Integer(signed=True, width=64)
    u64_int = Integer(signed=False, width=64)

    i128_int = Integer(signed=True, width=128)
    u128_int = Integer(signed=False, width=128)

    f32_num = Float(width=32)
    f64_num = Float()

    data_time_0 = DateTime()

#     fundamental = named("fundamental", Object("Fundamental"))

#     fundamental.event_name = named("event_name")
#     fundamental.event_category = named("event_category")

#     fundamental.inner = named("inner", Object("Inner"))
#     fundamental.inner.test = named("test", Float())

#     encoded_fps = named("encoded_fps", Float())
#     inferred_rendered_fps = named("inferred_rendered_fps", Float())

#     currency = named("currency", CStyleEnum(Currency))

#     i8_int = named("i8_int", Integer(signed=True, width=8))
#     u8_int = named("u8_int", Integer(signed=False, width=8))

#     i16_int = named("i16_int", Integer(signed=True, width=16))
#     u16_int = named("u16_int", Integer(signed=False, width=16))

#     i32_int = named("i32_int", Integer())
#     u32_int = named("u32_int", Integer(signed=False, width=32))

#     i64_int = named("i64_int", Integer(signed=True, width=64))
#     u64_int = named("u64_int", Integer(signed=False, width=64))

#     i128_int = named("i64_int", Integer(signed=True, width=128))
#     u128_int = named("u64_int", Integer(signed=False, width=128))

#     f32_num = named("f32_num", Float(width=32))
#     f64_num = named("f64_num", Float())

#     data_time_0 = named("date_time_0", DateTime())


if __name__ == "__main__":
    nestable_input_signal = NestableInputSignal()

    signal_schema = nestable_input_signal.to_schema()
    signal_patch_schema = nestable_input_signal.to_patch_schema()

    defs = signal_patch_schema["$defs"] | signal_schema["$defs"]
    # print(signal_patch_schema["$defs"].keys())
    # print("\n\n\n")
    # print(signal_schema["$defs"].keys())
    # print("\n\n\n")
    # print(defs.keys())

    # defs = signal_schema["$defs"]
    # signal_schema.pop("$defs")
    # signal_patch_schema.pop("$defs")

    # # input_signal_bag = signal_schema["properties"]
    # # signal_schema["input_signal_bag"] = "#def/input_signal_bag"

    # # input_signal_patch_bag = signal_schema["properties"]
    # # signal_patch_schema["input_signal_patch_bag"] = "#def/input_signal_patch_bag"

    defs = {
        "input_signal_bag": signal_schema,
        "input_signal_bag_patch": signal_patch_schema,
    } | defs
    

    result = {
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$comment": "InputSignalBagPatch and InputSignalBag",
        "$properties": {
            "input_signal_bag": "#/$defs/input_signal_bag",
            "input_signal_bag_patch": "#/$defs/input_signal_bag_patch"
        },
        "$defs": defs
    }

    print(json.dumps(result, indent=4), end='\n\n')

    # patch_schema = nestable_input_signal.to_patch_schema()
    # print(json.dumps(patch_schema, indent=4), end='\n\n')

    # schema = nestable_input_signal.to_schema()
    # print(json.dumps(schema, indent=4), end='\n\n')
