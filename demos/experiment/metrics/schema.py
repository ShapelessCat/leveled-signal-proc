from typing import final

from lsdl.lsp_model import CStyleEnum, Float, InputSchemaBase, LspEnumBase, String


# Customize our StrEnum subtype to regulate the value case issue
@final
class Currency(LspEnumBase):
    Unknown = "Unknown"
    Cny = "CNY"
    Euro = "EURO"
    Usd = "USD"


class InputSignal(InputSchemaBase):
    _timestamp_key = "timestamp"

    event_name = String()  # noqa: E221
    event_category = String()  # noqa: E221

    conviva_video_events_name = String()  # noqa: E221

    screen_id = String()  # noqa: E221
    page_id = String()  # noqa: E221

    inferred_player_state = String()  # noqa: E221

    encoded_fps = Float()  # noqa: E221
    inferred_rendered_fps = Float()  # noqa: E221

    currency = CStyleEnum(Currency)


input_signal = InputSignal()
