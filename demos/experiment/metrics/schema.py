from lsdl.prelude import InputSchemaBase, String, Float


class InputSignal(InputSchemaBase):
    _timestamp_key = 'timestamp'

    event_name                = String()  # noqa: E221
    event_category            = String()  # noqa: E221

    conviva_video_events_name = String()  # noqa: E221

    screen_id                 = String()  # noqa: E221
    page_id                   = String()  # noqa: E221

    inferred_player_state     = String()  # noqa: E221

    encoded_fps               = Float()   # noqa: E221
    inferred_rendered_fps     = Float()   # noqa: E221


input_signal = InputSignal()
