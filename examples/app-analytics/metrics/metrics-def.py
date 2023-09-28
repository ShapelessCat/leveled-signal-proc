from lsdl.schema import InputSchemaBase
from lsdl import print_ir_to_stdout

class Input(InputSchemaBase):
    _timestamp_key = "timestamp"

input = Input()


print_ir_to_stdout()