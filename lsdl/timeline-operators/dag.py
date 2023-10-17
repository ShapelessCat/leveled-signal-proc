from operators import *
from lsdl.schema import *
from lsdl.prelude import *
import yaml


class InputSignal(InputSchemaBase):
    pass


class DataSource(object):

    def __init__(self, data_source_config):
        self.data_source_config = data_source_config

    def get_input(self) -> InputSignal:
        input = InputSignal()
        input._timestamp_key = self.data_source_config.get("timePath")
        for field, field_type in self.data_source_config["format"]["schema"]["fields"].items():
            if field_type == "String":
                setattr(input, field, String())
            elif field_type == "Number":
                setattr(input, field, Float())
            elif field_type == "Boolean":
                setattr(input, field, Bool())
            else:
                raise Exception("Unknown field type: " + field_type)
        input.rebuild()
        return input


class Dag(object):
    timeline = None
    data_sources = {}

    def __init__(self, dag_config: dict):
        self.dag_config = dag_config

    def build_dag(self):
        for output_name, output_config in self.dag_config["outputs"].items():
            self.timeline = self._parse_operator(self.dag_config["dag"][output_config["timeline"].strip("$")])
            self.timeline.add_metric(output_name)

    def _parse_block(self, block_config):
        timeline = None
        if isinstance(block_config, str) and block_config.startswith("$"):
            timeline = self._parse_operator(self.dag_config["dag"][block_config.strip("$")])
        elif "op" in block_config:
            timeline = self._parse_operator(block_config)
        else:
            timeline = block_config
        return timeline

    def _parse_operator(self, timeline_config):
        op_name = timeline_config["op"]
        if op_name == "eventSourceTimeline":
            source = timeline_config["source"].strip("$")
            if source not in self.data_sources:
                data_source_config = self.dag_config["inputs"][source]
                self.data_sources[source] = DataSource(data_source_config).get_input()
            return self.data_sources[source]
        elif op_name == "count":
            timeline = self._parse_block(timeline_config["in"])
            return Count(timeline).process()
        elif op_name == "constant":
            return Constant().process(timeline_config["value"])
        elif op_name == "get":
            timeline = self._parse_block(timeline_config["in"])
            return Get(timeline).process(timeline_config["path"])
        elif op_name == "not":
            timeline = self._parse_block(timeline_config["in"])
            return Not(timeline).process()
        elif op_name == "durationTrue":
            timeline = self._parse_block(timeline_config["in"])
            return DurationTrueT(timeline).process()
        elif op_name == "and":
            args = []
            for config in timeline_config["args"]:
                timeline = self._parse_block(config)
                args.append(timeline)
            return And(args).process()
        elif op_name == "or":
            args = []
            for config in timeline_config["args"]:
                timeline = self._parse_block(config)
                args.append(timeline)
            return Or(args).process()
        elif op_name == "equals":
            args = []
            for config in (timeline_config["left"], timeline_config["right"]):
                timeline = self._parse_block(config)
                args.append(timeline)
            return Equals(args[0], args[1]).process()
        elif op_name in ("latestEventToState", "evaluateAt"):
            timeline = self._parse_block(timeline_config["in"])
            return timeline


if __name__ == "__main__":
    with open("playtime.yaml") as f:
        config = yaml.load(f, Loader=yaml.FullLoader)
    dag = Dag(config)
    dag.build_dag()
    print_ir_to_stdout()