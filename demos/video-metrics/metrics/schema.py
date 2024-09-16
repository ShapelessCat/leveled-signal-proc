from lsdl.lsp_model import Integer, SessionizedInputSchemaBase, String, named


class InputSignal(SessionizedInputSchemaBase):
    _timestamp_key = "timestamp"

    session_id   = named("sessionId")           # noqa: E221
    player_state = named("PlayerState")         # noqa: E221
    cdn          = named("CDN")                 # noqa: E221
    bit_rate     = named("BitRate", Integer())  # noqa: E221
    ev           = named("ev")                  # noqa: E221

    bit_rate_default = -1

    def create_epoch_signal(self):
        return self.session_id.clock()

    def create_session_signal(self):
        return self.session_id.count_changes()


input_signal = InputSignal()
