from lsdl.component import *
from lsdl.schema import *
from lsdl import measurement_config, print_ir_to_stdout

class Input(InputSchemaBase):
    player_state = named("newPlayerState", String())
    network      = named("newNetwork",     String())
    cdn          = named("newCdn",         String())
    user_action  = named("newUserAction",  String())

input = Input()
playtime = input.map(
    bind_var = "in",
    lambda_src= """
        in.player_state == "playting" &&
        in.cdn == "cdn1" &&
        in.network == "WIFI"
    """
).measure_duration_true()

measurement_config().add_metric("playTime", playtime).set_metrics_drain("json")

print_ir_to_stdout()