# LSDL

_The Leveled Signal Description Language (__LSDL__)_ is a DSL describing data logic for the
leveled-signal based data analytics system. The LSDL is built on top of Python3. We can use Python's
language feature to define schema, develop a high level module system and finally build a web-based
GUI for those most commonly used queries. This document is aiming to clarify the detailed design of
the LSDL.

The problem scope for __LSDL__ is:

- Define input schema.
- Define data logic.
- Configure the moment of interest merging policy, the policy about how to handle simultaneous events.
- Configure the metrics producing policy (e.g. realtime vs n minute(s)).

## How to use

Each module of __LSDL__ has a well define package `__init__.py`, and all the functionalities for public
use are imported there. The recommended usage for newcomers:

- `from lsdl import *`:
  When you want to use `processing_config`, `measurement_config`, and `print_ir_to_stdout`.
  Usually, you only need these in your project entry point file.

- `from lsdl.lsp_model import *`:
  When you need LSP model fundamental components. 
  Mostly, you need these when defining the schema for input signals.

- `from lsdl.processors import *`:
  When you need signal processors.
  For experienced users, it's better to point out the specific processor(s), rather than using `*`.

- `from lsdl.measurements import *`:
  When you need measurements.
  For experienced users, it's better to point out the specific measurement(s), rather than using `*`.

## Useful Info

- [The LSDL Specification](https://conviva.atlassian.net/wiki/spaces/~712020f765b3b30d0e446096dbfeb73b527a21/pages/1903166610/The+LSDL+Specification)