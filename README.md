# AoC Lang

Almost Operational Coding Language is my attempt at creating a programming
language, including the tools around it, that is usable enough to solve
Advent of Code 2024.

For the language to be almost operational it should have:

- [x] an interpreter,
- [ ] syntax highlighting,
- [ ] basic LSP.

## Usage

1. Clone this repository:
   ```sh
   git clone https://github.com/viddrobnic/aoc-lang.git
   ```
2. Build and install the interpreter:
   ```sh
   cd aoc-lang
   cargo install --path .
   ```
3. Run some code:
   ```sh
   aoc-lang examples/hello_world.aoc
   ```

## Language features

AoC language supports the following features:

- integers
- floats
- booleans
- strings:
- arrays
- hash maps
- arithmetic operations (`+`, `-`, `*`, `/`, `%`)
- bit-wise operations (`&`, `|`, `!`)
- comparison operations (`<`, `>`, `<=`, `>=`, `==`, `!=`)
- logical operations (`!`, `&`, `|`)
- variables
- multi variable assignment (`[a, b] = [10, 20]`)
- if/else statements
- while loop
- for loop
- break
- continue
- functions
- comments
- stdin, stdout
- imports
- error reporting with line numbers

For more detailed overview of the syntax, see `examples` directory,
which contains examples of code with comments.

## Wishlist:

### Syntax highlighting

Since I am using neovim as my editor, the easiest way to get syntax highlighting
is with treesitter:

- [ ] define treesitter grammar
- [ ] write highlighting queries
- [ ] update neovim config

### LSP

I didn't look into how LSP implementation is done yet, so I don't know how hard
it will be. But I can still write a wishlist :)

- [ ] diagnostics
- [ ] go to definition
- [ ] basic autocomplete
- [ ] formatting
