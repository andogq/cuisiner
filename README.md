# cuisiner

***Cook up some conversions to and from C-style binary layouts.***

Cuisiner provides traits and macros to create idiomatic Rust structures that can be seamlessly
converted to and from C-style binary representations, even if it has a different layout.

## Overview

Cuisiner centres on the derivable `Cuisiner` trait, which provides the 'raw' serialised type (via
the `Raw` associated type), and methods used to serialise and deserialise from the raw value. The
raw representation must align with the C representation of the structure, whilst the `try_from_raw`
and `try_to_raw` methods can handle validation when converting to and from the idiomatic Rust
representation.

## Example

See [`sqlite-header.rs`](./tests/sqlite-header.rs) for an example.

## Todo

- [ ] Mirror `zerocopy`'s API for reading to/from bytes (`read_prefix`, `read_suffix`, etc)

- [ ] Add support for reading from `Readable`

