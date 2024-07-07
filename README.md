# AoC Lang

Almost Operational Coding Language is my attempt at creating a programming
language, including the tools around it, that is usable enough to solve
Advent of Code 2024.

For the language to be almost operational it should have:

- [x] an interpreter,
- [x] syntax highlighting,
- [ ] basic LSP.

## Usage

### Interpreter setup

Rust is required to build the interpreter.

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

### Syntax highlighting

Syntax highlighting is implemented with [tree sitter](https://tree-sitter.github.io/tree-sitter/).
I am using neovim as my editor (btw) and this is the standard way of doing it.
If you are using some other editor, you are on your own...

1. Add the aoc tree sitter parser to your config:
   ```lua
   local parser_config = require('nvim-treesitter.parsers').get_parser_configs()
   parser_config.aoc = {
     install_info = {
       url = 'https://github.com/viddrobnic/aoc-lang.git',
       files = { 'tree-sitter-aoc/src/parser.c' },
     },
     filetype = 'aoc',
   }
   ```
2. Open neovim and install the parser
   ```
   :TSInstall aoc
   ```
3. Configure automatic aoc file type detection:
   ```lua
   vim.filetype.add({
     extension = {
       aoc = 'aoc',
     },
   })
   ```
4. Add highlighting queries by copying
   [this file](https://github.com/viddrobnic/aoc-lang/blob/master/tree-sitter-aoc/queries/highlights.scm)
   from [the repo](https://github.com/viddrobnic/aoc-lang) to
   ```
   ~/.config/nvim/queries/aoc/
   ```
5. Restart neovim and open an `.aoc` file :)

## Language features

AoC language supports the following features:

- integers
- floats
- booleans
- strings
- characters
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

### LSP

I didn't look into how LSP implementation is done yet, so I don't know how hard
it will be. But I can still write a wishlist :)

- [ ] diagnostics
- [ ] go to definition
- [ ] basic autocomplete
- [ ] formatting
