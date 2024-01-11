from lsdl.prelude import named, Integer, SessionizedInputSchemaBase, String


class InputSignal(SessionizedInputSchemaBase):
    _timestamp_key = "timestamp"

    session_id   = named("sessionId",   String())   # noqa: E221
    player_state = named("PlayerState", String())   # noqa: E221
    cdn          = named("CDN",         String())   # noqa: E221
    bit_rate     = named("BitRate",     Integer())  # noqa: E221
    ev           = named("ev",          String())   # noqa: E221

    bit_rate_default = -1

    def create_epoch_signal(self):
        return self.session_id.clock()

    def create_session_signal(self):
        return self.session_id.count_changes()


input_signal = InputSignal()
