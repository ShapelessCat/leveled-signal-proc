from lsdl.component import *
from lsdl.schema import *
from lsdl import measurement_config, print_ir_to_stdout

class Input(InputSchemaBase):
    player_state = named("newPlayerState", String())
    network      = named("newNetwork",     String())
    cdn          = named("newCdn",         String())
    user_action  = named("newUserAction",  String())

input = Input()

((input.player_state == "playing") & (input.cdn == "cdn1") & (input.network == "WIFI"))\
    .measure_duration_true() \
    .add_metric("playtime")

print_ir_to_stdout()