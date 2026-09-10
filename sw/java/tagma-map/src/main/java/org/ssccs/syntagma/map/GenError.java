package org.ssccs.syntagma.map;

/**
 * The failure taxonomy of coordinate generation.
 *
 * <p>Every strategy rejects the empty key ({@link #EMPTY_KEY}); a strategy with
 * a fixed depth can also report a key longer than it can fold
 * ({@link #KEY_TOO_LONG}). The Java strategy surface reports these failures as
 * {@code Optional.empty()}, the mapping of the C++ {@code std::optional}
 * return, mirroring the C++ port, which declares the same enum and also
 * reports failure through the optional return.
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
