#pragma once

// CoordKV2: a 2-byte-key hashless store. The Rust reference backs this
// with the dense CoordSpace2 (119 MB single allocation); this C++ port
// uses the lazy CoordSpaceN<2> tree instead, keeping the same API and
// behavior with memory proportional to entries. Mirrors the Rust
// CoordKV2 in sw/rust/kv/src/coord_kv2.rs.

#include "tagma_kv/coord_kv_n.h"

namespace tagma_kv {

using CoordKV2 = CoordKVN<2>;

}  // namespace tagma_kv
