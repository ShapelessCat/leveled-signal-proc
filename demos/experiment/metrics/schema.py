from lsdl.prelude import *


class InputSignal(InputSchemaBase):
    _timestamp_key        = 'timestamp'

    inferred_player_state = String()

    encoded_fps           = Float()

    inferred_rendered_fps = Float()


input_signal = InputSignal()
