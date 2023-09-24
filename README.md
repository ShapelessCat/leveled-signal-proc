# LSP Time-state Analytics Platform

The experimental time-state analytics platform written in Rust.

## Dependencies

- Latest Rust compiler (MSRV is still unknown but _rustc_ 1.71 successfully compiles)
- Python 3 (For LSDL)

## Build

- For hand-written examples, simply run
  
```shell
cargo build --examples
```

- For LSDL examples

```shell
cargo build --package=lsp-codegen-test
```

## Examples

There are hand-written examples in the `lsp-runtime/examples/` directory.

There are some examples written in `LDSL`, the DSL we use in the LSP framework. `lsdl/examples/`. As we are moving forward quickly, more LSDL examples will be added.

### LDSL

For LDSL examples, check `ldsl/examples/` directory. All the Python source code are LDSL and JSON files are the IRs generated from them.

#### Trying out examples written in LSDL

Currently we are able to run the LSDL written data logic reading from and writing to files on disk. 
To try out that,

```shell
cargo build         # for release build, add --release parameter to the command
target/release/cidr # For CIDR example, replace cidr with other example names to try out other examples
```

#### Visualize LSDL IR as Computation Graph

You can use the `lsp-ir-to-dot-graph` program to visualize the LSP-IR. \
For examples in `ldsl/examples/`, you can find the computation graph visualization at `assets/lsdl-example-svg` directory. \
For the generated code of all LDSL examples, please check the `assets/lsdl-example-expanded` directory.

## Useful links

- General Idea: [Introduction to leveled signal abstraction](https://conviva.atlassian.net/wiki/spaces/~712020f765b3b30d0e446096dbfeb73b527a21/pages/1879934386/LSP+High+Level+Design)
- Writting data logic: [LDSL Introduction](https://conviva.atlassian.net/wiki/spaces/~712020f765b3b30d0e446096dbfeb73b527a21/pages/1903166610/The+LSDL+Specification)