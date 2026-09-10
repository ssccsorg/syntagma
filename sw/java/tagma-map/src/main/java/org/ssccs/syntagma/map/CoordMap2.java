package org.ssccs.syntagma.map;

/**
 * A 2-byte-key map, the Java counterpart of the C++ alias
 * {@code using CoordMap2 = CoordMapN<2>} and of the Rust {@code CoordMap2}
 * struct.
 *
 * <p>The Rust reference backs this type with the dense {@code CoordSpace2}
 * (a 119 MB single allocation) for O(1) lookup. The C++ port keeps the API and
 * the behavior but backs the map with the lazy {@code CoordSpaceN<2>} tree, so
 * memory stays proportional to the entries; the Java core has no dense
 * depth-2 space either, its only dense space being the depth-3 file-mapped
 * {@code CoordSpaceM}. This port therefore follows the C++ deviation, and
 * {@link CoordMapN} fixed at depth 2 is the whole implementation.
 *
 * <p>Port of the C++ {@code tagma_map::CoordMap2} in
 * {@code sw/cpp/tagma_map/include/tagma_map/coord_map2.h}; the underlying
 * behavior mirrors the Rust {@code CoordMap2} in
 * {@code sw/rust/map/src/coord_map2.rs}.
 */
public final class CoordMap2 extends CoordMapN {

    /** Creates an empty store whose keys are exactly 2 UTF-8 bytes long. */
    public CoordMap2() {
        super(2);
    }
}
