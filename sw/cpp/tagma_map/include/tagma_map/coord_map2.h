#pragma once

// CoordMap2: a 2-byte-key hashless map. The Rust reference backs this
// with the dense CoordSpace2 (119 MB single allocation); this C++ port
// uses the lazy CoordSpaceN<2> tree instead, keeping the same API and
// behavior with memory proportional to entries. Mirrors the Rust
// CoordMap2 in sw/rust/map/src/coord_map2.rs.

#include "tagma_map/coord_map_n.h"

namespace tagma_map {

using CoordMap2 = CoordMapN<2>;

}  // namespace tagma_map
