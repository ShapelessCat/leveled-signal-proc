class _MeasurementConfiguration(object):
    """
        The configuration for measurement policy. 
        In LSP, there are two method to trigger a measurement: 
            1. triggered by a input event;
            2. triggered by a signal edge; 
    """
    def __init__(self):
        self._measure_at_event_lambda = "|_| true"
        self._measure_on_edge = None
        self._measure_side_flag = None
        self._metrics_drain = "json"
        self._output_schema = {}
    def set_measure_at_filter(self, lambda_src: str):
        """
            Set the rule for event triggered measurement. 
        """
        self._measure_at_event_lambda = lambda_src
        return self
    def enable_measure_for_event(self):
        """
            Enable measurement on every input event behavior
        """
        self._measure_at_event_lambda = "|_| true"
        return self
    def disable_measure_for_event(self):
        """
            Prevent measurement on any input event
        """
        self._measure_at_event_lambda = "|_| false" 
        return self
    def set_trigger_signal(self, signal):
        """
            Set the measurement control signal, this signal will trigger a measurement when the value of the signal gets changed
        """
        self._measure_on_edge = signal
        return self
    def set_limit_side_signal(self, signal):
        """
            Configure the limit side for the measurement. Normally, LSP uses the right-side limit for measurements. 
            While for some speccial case, for example, the summary for the last session, we should use the left-side limit
            semantics. And this is the signal that switches the limit-side semantics during the run-time.
        """
        self._measure_side_flag = signal
        return self
    def set_metrics_drain(self, fmt: str):
        """
            Configure what format we want the LSP system produce. 
            Note: Currently JSON is the only valid option.
        """
        self._metrics_drain = fmt
        return self
    def add_metric(self, key, measurement, typename = "_"):
        """
            Declare a metric for output.
        """
        if typename == "_":
            typename = measurement.get_rust_type_name()
        if typename == "_":
            raise "Type name must specified for a metric. Consider call `.annotate_type(<type-name>)` to manually annotate signal's type"
        self._output_schema[key] = {
            "source": measurement.get_id(),
            "type": typename
        }
        return self
    def to_dict(self):
        """
            Make the measurement policy data structure into a dictionary that can be JSONified.
        """
        ret = {
            "measure_at_event_filter": self._measure_at_event_lambda,
            "metrics_drain": self._metrics_drain,
            "output_schema": self._output_schema,
        }
        if self._measure_on_edge is not None:
            ret["measure_trigger_signal"] = self._measure_on_edge.get_id()
        if self._measure_side_flag is not None:
            ret["measure_left_side_limit_signal"] = self._measure_side_flag.get_id()
        return ret
    
def _make_measurement_configuration():
    config = _MeasurementConfiguration()
    def measurement_config() -> _MeasurementConfiguration:
        nonlocal config
        return config
    return measurement_config
    
measurement_config = _make_measurement_configuration()