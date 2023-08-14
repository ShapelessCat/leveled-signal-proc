class _MeasurementConfiguration(object):
    def __init__(self):
        self._measure_at_event_lambda = "|_| true"
        self._measure_periodically_interval = -1
        self._metrics_drain = "json"
        self._output_schema = {}
    def set_measure_at_filter(self, lambda_src: str):
        self._measure_at_event_lambda = lambda_src
        return self
    def set_measure_periodically_interval(self, interval: int):
        self._measure_periodically_interval = interval
        return self
    def set_metrics_drain(self, fmt: str):
        self._metrics_drain = fmt
        return self
    def add_metric(self, key, measurement):
        self._output_schema[key] = measurement.get_id()
        return self
    def to_dict(self):
        ret = {
            "measure_at_event_filter": self._measure_at_event_lambda,
            "measure_periodically_interval": self._measure_periodically_interval,
            "metrics_drain": self._metrics_drain,
            "output_schema": self._output_schema,
        }
        return ret
    
def _make_measurement_configuration():
    config = _MeasurementConfiguration()
    def measurement_config() -> _MeasurementConfiguration:
        nonlocal config
        return config
    return measurement_config
    
measurement_config = _make_measurement_configuration()