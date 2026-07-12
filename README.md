# Tagma

Content-addressable structural primitive defined by the Hangul syllabic space.

This repository contains the reference implementation of Tagma — a production-level,
no_std Rust foundation library (Apache 2.0) that replaces hash-based identity generation
with a combinational decoder over a fixed 16-bit coordinate space.

## Repository structure

- `docs/` — White paper (`wp.qmd`) and master document (`index.qmd`)
- `sw/rust/` — Rust workspace
  - `core/` — `tagma-core`: TagmaCoord, TagmaMap, TagmaSet, TagmaTimeIndex
  - `base11172/` — Tagma native serialization format
- `hw/` — Verilog decoder, XIF interface, 3D SRAM array (future)
- `poc/` — Archived early proofs of concept (base11172 standalone, nex-tagma)

## License

Apache 2.0
