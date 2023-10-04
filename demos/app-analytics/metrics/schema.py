from lsdl.schema import DateTime, InputSchemaBase, named, String, volatile 

class Input(InputSchemaBase):
    _timestamp_key             = 'timestamp'
    event_name                 = named('event_name')
    event_category             = named('event_category')
    platform                   = named('platform')
    page_id                    = named('page_id')
    load_start                 = named('load_start')
    load_end                   = named('load_end')
    conviva_video_events_name  = named('conviva_video_events_name')
    reponse_code               = named('reponse_code')
    network_request_duration   = named('network_request_duration')
    app_startup_start          = named('app_startup_start')
    app_startup_end            = named('app_startup_enda')
    app_startup_previous_exist = named('app_startup_previous_exist')

    start_ts       = volatile(DateTime())
    end_ts         = volatile(DateTime())
    prev_exist     = volatile(String())