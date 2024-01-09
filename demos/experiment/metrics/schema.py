from lsdl.prelude import InputSchemaBase, String, Float


class InputSignal(InputSchemaBase):
    _timestamp_key = 'timestamp'

    event_name = String()
    event_category = String()

    conviva_video_events_name = String()

    screen_id = String()
    page_id = String()

    inferred_player_state = String()

    encoded_fps = Float()
    inferred_rendered_fps = Float()


input_signal = InputSignal()
