# 3. Reading the input

Crate `guaraci-input`.

## Parser

A hand-written, line-oriented recursive-descent parser (`parser.rs`). Every
node of the tree keeps a `Span` (file, start, end), used in every message.

Relevant lexical rules:

- A number may be followed, on the same line, by a unit, with or without a
  space. A unit starts with a letter, `µ`, `°` or `%`.
- The exponent `e` is part of the number only if digits follow it.
- Unquoted words accept letters, digits and `_ - . / +`, which covers names
  such as `Tc-99m` and `dose.mhd`.
- `131I` and `99mTc` are read by the parser as a number followed by a unit.
  The `nuclide` key rebuilds the text before interpreting the name.

## `include`

Resolution is in `source.rs`:

- If the name has no extension, `name.gci` is tried first, then `name`.
- The path is relative to the file that contains the `include`.
- Files already read are skipped (diamond includes).
- An include stack detects cycles.

Built-in files (the material database) are loaded before the user's file and
marked as *built-in*. User definitions with the same name replace them.

## From the tree to the configuration

The section reader (`reader.rs`) offers typed reads (`quantity`, `vec3`,
`choice`, `text_list`…) and:

- records every key read; the remaining ones become "unknown setting", with a
  suggestion based on Jaro-Winkler similarity;
- records every default value applied (`DefaultNote`);
- accumulates errors instead of stopping at the first one.

Building the configuration (`config.rs`) also:

- resolves the `same_as` chains of materials, detecting cycles;
- checks every cross reference (volumes, materials, parents) and parent cycles;
- does not repeat errors in cascade: if the section of a volume has an error,
  references to that volume are not reported again.
