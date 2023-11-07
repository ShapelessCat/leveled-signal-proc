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
    output_streams = []
    processed_node = {}

    def __init__(self, dag_config: dict):
        self.dag_config = dag_config

    def build_dag(self):
        for _, output_config in self.dag_config["outputs"].items():
            timeline_name = output_config["timeline"].strip("$")
            self.output_streams.append(self._parse_operator(self.dag_config["dag"][timeline_name], timeline_name))

    def _parse_block(self, block_config):
        timeline = None
        if isinstance(block_config, str) and block_config.startswith("$"):
            timeline = self._parse_operator(self.dag_config["dag"][block_config.strip("$")], block_config.strip("$"))
        elif "op" in block_config:
            timeline = self._parse_operator(block_config)
        else:
            timeline = block_config
        return timeline
    
    def _parse_kary_args(self, timeline_config):
        args = []
        for config in timeline_config["args"]:
            timeline = self._parse_block(config)
            args.append(timeline)
        return args

    def _parse_binary_args(self, timeline_config):
        left_timeline = self._parse_block(timeline_config["left"])
        right_timeline = self._parse_block(timeline_config["right"])
        return left_timeline, right_timeline

    def _parse_operator(self, timeline_config, timeline_name = None):
        if timeline_name in self.processed_node:
            return self.processed_node[timeline_name]
        output_timeline = None
        op_name = timeline_config["op"]
        if op_name == "eventSourceTimeline":
            source = timeline_config["source"].strip("$")
            data_source_config = self.dag_config["inputs"][source]
            output_timeline = DataSource(data_source_config).get_input()
        elif op_name == "count":
            timeline = self._parse_block(timeline_config["in"])
            output_timeline = Count(timeline).process().peek().add_metric(timeline_name)
        elif op_name == "constant":
            output_timeline = Constant().process(timeline_config["value"])
        elif op_name == "get":
            timeline = self._parse_block(timeline_config["in"])
            output_timeline = Get(timeline).process(timeline_config["path"])
        elif op_name == "not":
            timeline = self._parse_block(timeline_config["in"])
            output_timeline = Not(timeline).process()
        elif op_name == "durationTrue":
            timeline = self._parse_block(timeline_config["in"])
            output_timeline = DurationTrueT(timeline).process().add_metric(timeline_name)
        elif op_name == "and":
            args = self._parse_kary_args(timeline_config)
            output_timeline = And(args).process()
        elif op_name in ("or", "mergeEvents"):
            args = self._parse_kary_args(timeline_config)
            output_timeline = Or(args).process()
        elif op_name == "equals":
            left, right = self._parse_binary_args(timeline_config)
            output_timeline = Equals(left, right).process()
        elif op_name == "latestEventToState":
            output_timeline = self._parse_block(timeline_config["in"])
        elif op_name == "evaluateAt":
            output_timeline = self._parse_block(timeline_config["in"])
        elif op_name == "makeStruct":
            for _, config in timeline_config["fields"].items():
                timeline = self._parse_block(config)
            output_timeline = timeline
        elif op_name == "filterByValue":
            timeline = self._parse_block(timeline_config["in"])
            output_timeline = FilterByValue(timeline).process(timeline_config["values"])
        elif op_name in ("greaterThan", "greaterThanOrEqualTo", "lessThan", "lessThanOrEqualTo"):
            left, right = self._parse_binary_args(timeline_config)
            output_timeline = Inequality(left, right).process(op_name)
        elif op_name == "add":
            args = self._parse_kary_args(timeline_config)
            output_timeline = Add(args).process()
        elif op_name == "multiply":
            args = self._parse_kary_args(timeline_config)
            output_timeline = Multiply(args).process()
        elif op_name == "substract":
            left, right = self._parse_binary_args(timeline_config)
            output_timeline = Substract(left, right).process()
        elif op_name == "divide":
            left, right = self._parse_binary_args(timeline_config)
            output_timeline = Divide(left, right).process()
        elif op_name == "any":
            timeline = self._parse_block(timeline_config["in"])
            output_timeline = Any(timeline).process()

        self.processed_node[timeline_name] = output_timeline
        return output_timeline


if __name__ == "__main__":
    with open("playtime.yaml") as f:
        config = yaml.load(f, Loader=yaml.FullLoader)
    dag = Dag(config)
    dag.build_dag()
    print_ir_to_stdout()