# AoC Lang

Almost Operational Coding Language is my attempt at creating a programming
language, including the tools around it, that is usable enough to solve
Advent of Code 2024.

For the language to be almost operational it should have:

- an interpreter,
- syntax highlighting,
- basic LSP.

## Wishlist:

### Language features:

- [ ] integers
  - from float
  - from string
- [ ] floats
  - from int
  - from string
- [ ] booleans
  - from string
- [ ] strings
  - concatenate
  - append
  - split
- [ ] arrays
  - concatenate
  - push
  - pop
  - unpacking via assignment: `[a, b] = [42, "foo"]`
- [ ] hash maps
  - add
  - remove
- [ ] arithmetic operations (`+`, `-`, `*`, `/`, `%`)
- [ ] bit-wise operations (`&`, `|`, `!`)
- [ ] comparison operations (`<`, `>`, `<=`, `>=`, `==`, `!=`)
- [ ] logical operations (`!`, `&&`, `||`)
- [ ] variables
- [ ] if/else statements
- [ ] while loop
- [ ] for loop
- [ ] break
- [ ] continue
- [ ] functions
  - return
  - recursion
- [ ] comments
- [ ] stdin, stdout
- [ ] imports
- [ ] error reporting with line numbers

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
