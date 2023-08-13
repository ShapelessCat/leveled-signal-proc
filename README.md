# LSP Time-state Analyitcs Platform

The experimential time-state analytics platform written in Rust.

# Dependecies

- Latest Rust compiler (MSRV is still unknown)
- Python 3 (For LSDL)

# Build

- For hand-written examples, simply run
  
```
cargo build --examples
```

- For LSDL examples

```
cargo build --package=lsp-codegen-test
```

# Examples

There are hand-written examples in `lsp-runtime/examples/` directory. 

There are some examples written in `LDSL`, the DSL we using in the LSP framework. `lsdl/examples/`. As we are moving forward quickly, more LSDL examples will be added.

# LDSL

For LDSL examples, check `ldsl/examples/` directory. All the python source code are LDSL and JSON file are the IRs generated from them.

# Useful links

- General Idea: [Introduction to leveled signal abstraction](https://conviva.atlassian.net/wiki/spaces/~712020f765b3b30d0e446096dbfeb73b527a21/pages/1879934386/LSP+High+Level+Design)
- Writting data logic: [LDSL Introduction](https://conviva.atlassian.net/wiki/spaces/~712020f765b3b30d0e446096dbfeb73b527a21/pages/1903166610/The+LSDL+Specification)