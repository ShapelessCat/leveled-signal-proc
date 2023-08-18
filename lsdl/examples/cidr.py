from lsdl.schema import *
from lsdl.modules import *
from lsdl import print_ir_to_stdout

class Input(InputSchemaBase):
    player_state = named("newPlayerState", String())
    network      = named("newNetwork",     String())
    cdn          = named("newCdn",         String())
    user_action  = named("newUserAction",  String())

input = Input()

target = has_been_true(input.user_action == "play") &\
      ~has_been_true(input.user_action == "seek", 5_000_000_000) &\
        (input.player_state == "buffer") &\
        (input.cdn == "cdn1")
target.measure_duration_true().add_metric("totalPlayTime")

print_ir_to_stdout()