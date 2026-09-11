package org.ssccs.syntagma.map;

/**
 * The failure taxonomy of coordinate generation, kept in the public surface to
 * mirror the reference taxonomy: no type or method of this module produces or
 * consumes a {@code GenError}.
 *
 * <p>The strategy surface reports a rejected key as {@code Optional.empty()},
 * the mapping of the C++ {@code std::optional} return, and a key of the wrong
 * length throws {@link IllegalArgumentException} from the member that detects
 * it. The two members therefore name the failures the references classify, not
 * failures this port returns: the empty key, and the oversize key that the Rust
 * {@code CoordKey} reports as {@code KeyTooLong}.
 *
 * <p>Port of the C++ {@code tagma_map::GenError} in
 * {@code sw/cpp/tagma_map/include/tagma_map/coord_gen.h}; the underlying
 * behavior mirrors the Rust {@code GenError} in
 * {@code sw/rust/map/src/coord_gen.rs}. The Rust variant
 * {@code KeyTooLong { max_len, actual_len }} carries the two lengths; the C++
 * enum does not, and Java follows the C++ shape.
 */
public enum GenError {

    /** The key is empty; no valid path can be produced. */
    EMPTY_KEY,

    /** The key exceeds the maximum length supported by the strategy. */
    KEY_TOO_LONG
}
