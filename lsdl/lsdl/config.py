class _MeasurementConfiguration(object):
    def __init__(self):
        self._measure_at_event_lambda = "|_| true"
        self._measure_on_edge = None
        self._measure_periodically_interval = -1
        self._metrics_drain = "json"
        self._output_schema = {}
    def set_measure_at_filter(self, lambda_src: str):
        self._measure_at_event_lambda = lambda_src
        return self
    def enable_measure_for_event(self):
        self._measure_at_event_lambda = "|_| true"
        return self
    def disable_measure_for_event(self):
        self._measure_at_event_lambda = "|_| false" 
        return self
    def set_trigger_signal(self, signal):
        self._measure_on_edge = signal
        return self
    def set_metrics_drain(self, fmt: str):
        self._metrics_drain = fmt
        return self
    def add_metric(self, key, measurement, typename = "_"):
        if typename == "_":
            typename = measurement.get_rust_type_name()
        if typename == "_":
            raise "Type name must specified for a metric"
        self._output_schema[key] = {
            "source": measurement.get_id(),
            "type": typename
        }
        return self
    def to_dict(self):
        ret = {
            "measure_at_event_filter": self._measure_at_event_lambda,
            #"measure_periodically_interval": self._measure_periodically_interval,
            "metrics_drain": self._metrics_drain,
            "output_schema": self._output_schema,
        }
        if self._measure_on_edge is not None:
            ret["measure_trigger_signal"] = self._measure_on_edge.get_id()
        return ret
    
def _make_measurement_configuration():
    config = _MeasurementConfiguration()
    def measurement_config() -> _MeasurementConfiguration:
        nonlocal config
        return config
    return measurement_config
    
measurement_config = _make_measurement_configuration()