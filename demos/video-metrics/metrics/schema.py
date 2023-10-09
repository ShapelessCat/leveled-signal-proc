from lsdl.prelude import *


class Input(SessionizedInputSchemaBase):
    _timestamp_key = "timestamp"
    session_id     = named("sessionId",   String())
    player_state   = named("PlayerState", String())
    cdn            = named("CDN",         String())
    bit_rate       = named("BitRate",     Integer())
    ev             = named("ev",          String())

    bit_rate_default = -1

    def create_epoch_signal(self):
        return self.session_id.clock()

    def create_session_signal(self):
        return self.session_id.count_changes()
