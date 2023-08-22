use std::{fmt::Write, io::{Read, BufReader, BufRead}, path::Path, collections::HashMap, fs::File};

use anyhow::Error;
use lsp_ir::{LspIr, NodeInput, DebugInfo};

fn render_upstream_refs(input: &NodeInput) -> Vec<String> {
    let upstream = match input {
        NodeInput::Component { id } => {
            vec![format!("node_{}:output", id)]
        }
        NodeInput::Constant { value, .. } => {
            vec![format!("Const_{}", value)]
        }
        NodeInput::InputBag => {
            vec!["input".to_string()]
        }
        NodeInput::InputSignal { id } => {
            vec![format!("input:{}", id)]
        }
        NodeInput::Tuple { values } => values
            .iter()
            .map(|node| render_upstream_refs(node).into_iter())
            .flatten()
            .collect(),
    };
    upstream
}
fn visualize_lsp_ir<R: Read>(reader: R) -> Result<(), Error> {
    let ir: LspIr = serde_json::from_reader(reader)?;

    println!("digraph{{\n\trankdir=LR;");

    println!("\t{{");

    let mut schema_node = String::new();
    for (name, tn) in ir.schema.members.iter() {
        write!(&mut schema_node, "<{name}>{name}|", name = name)?;
        write!(
            &mut schema_node,
            "<{name}>&lt;clk&gt;|",
            name = tn.clock_companion
        )?;
    }
    schema_node.pop();

    println!(
        "\t\tinput[shape=record;style=filled;label=\"{schema}\"]",
        schema = schema_node
    );

    let mut source_code_list = HashMap::<_, Vec<_>>::new();

    for node in ir.nodes.iter() {
        if let Some(DebugInfo{ file, .. }) = node.debug_info.as_ref() {
            if let Ok(fp) = File::open(file) {
                let reader = BufReader::new(fp);
                let file : &Path = file.as_ref();
                if let Some(filename) = file.file_name().map(|s| s.to_string_lossy()) {
                    source_code_list.entry(filename.to_string()).or_insert_with(|| {
                        reader.lines().filter_map(|l| l.ok()).collect()
                    });
                }
            }
        }
        let mut ins = String::new();
        for (id, _input) in node.upstreams.iter().enumerate() {
            write!(&mut ins, "<input{id}>[{id}]|", id = id).unwrap();
        }
        ins.pop();
        let namespace = {
            if let Some(pos) = node.namespace.rfind("::") {
                node.namespace[pos + 2..].to_string()
            } else {
                node.namespace.clone()
            }
        };
        println!("\t\tnode_{id}[shape=record;label=\"{{{{{ins}}}|<output>#{id}:{namespace}}}\";tooltip=\"{decl}\"];",
            id = node.id,
            ins = ins,
            namespace = namespace,
            decl = node.node_decl.replace('"', "\\\"")
        );
    }
    for (metric_name, _) in ir.measurement_policy.output_schema.iter() {
        println!(
            "\t\toutput_{name}[shape=box;style=filled;fillcolor=gray;label=\"{name}\"]",
            name = metric_name
        );
    }
    println!("\t}}");

    for node in ir.nodes.iter() {
        for (id, input) in node.upstreams.iter().enumerate() {
            let input_ref = format!("node_{}:input{}", node.id, id);
            let upstreams = render_upstream_refs(input);
            for upstream in upstreams {
                println!("\t{} -> {};", upstream, input_ref);
            }
        }
    }

    for (metric_name, upstream) in ir.measurement_policy.output_schema {
        let upstreams = render_upstream_refs(&upstream.source);
        for upstream in upstreams {
            println!("\t{} -> output_{};", upstream, metric_name);
        }
    }
    let mut subgraphs = std::collections::HashMap::<_, (String, i32, Vec<_>)>::new();
    for node in ir.nodes.iter() {
        if let Some(di) = &node.debug_info {
            let file: &Path = di.file.as_ref();
            let name = file.file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or("<unknown>".to_string());
            let key = format!("{}:{}", name, di.line);
            subgraphs
                .entry(key)
                .or_insert_with(|| (name, di.line, Vec::new()))
                .2
                .push(format!("node_{}", node.id));
        }
    }
    for (id, (label, (file, line, subgraph))) in subgraphs.iter().enumerate() {
        println!("\tsubgraph sg_{} {{", id);
        println!("\t\tcluster = true;");
        println!("\t\tlabel = \"{label}\";", label = label);
        if let Some(src_list) = source_code_list.get(file) {
            if *line > 0 {
                let line = *line as usize - 1;
                if let Some(line) = src_list.get(line) {
                    println!("\t\ttooltip = {code}", code = serde_json::to_string(line).unwrap());
                }
            }
        }
        for node in subgraph {
            println!("\t\t{};", node);
        }
        println!("\t}}");
    }
    println!("}}");
    Ok(())
}

fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.is_empty() {
        visualize_lsp_ir(std::io::stdin()).unwrap();
    } else {
        visualize_lsp_ir(std::fs::File::open(&args[0]).unwrap()).unwrap();
    }
}
