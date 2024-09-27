from lsdl.lsp_model import InputSchemaBase, String, volatile


class InputSignal(InputSchemaBase):
    _timestamp_key = "timestamp"

    event_name = String()  # noqa: E221
    event_category = String()  # noqa: E221

    platform = String()  # noqa: E221

    page_id = String()  # noqa: E221
    screen_id = String()  # noqa: E221

    load_start = String()  # noqa: E221
    load_end = String()  # noqa: E221

    conviva_video_events_name = String()  # noqa: E221

    response_code = String()  # noqa: E221
    network_request_duration = String()  # noqa: E221

    app_startup_start = volatile(String())  # noqa: E221
    app_startup_end = volatile(String())  # noqa: E221
    app_startup_previous_exist = volatile(String())  # noqa: E221


input_signal = InputSignal()
