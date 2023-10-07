from lsdl.schema import InputSchemaBase, String, volatile 

class Input(InputSchemaBase):
    _timestamp_key             = 'timestamp'

    event_name                 = String()
    event_category             = String()

    platform                   = String()

    page_id                    = String()
    screen_id                  = String()

    load_start                 = String()
    load_end                   = String()

    conviva_video_events_name  = String()

    response_code              = String()
    network_request_duration   = String()

    app_startup_start          = volatile(String())
    app_startup_end            = volatile(String())
    app_startup_previous_exist = volatile(String())

input = Input()
