package org.ssccs.syntagma.map;

import java.util.Optional;

/**
 * The core string-key operations of a coordinate map, mirroring the Rust
 * {@code CoordMap} trait and the C++ method set.
 *
 * <p>The interface carries the same member set as
 * {@code java.util.HashMap} where applicable: {@link #insert}, {@link #get},
 * {@link #remove}, {@link #containsKey}, {@link #len}, {@link #isEmpty} and
 * {@link #clear}. No hashing is involved at any point.
 *
 * <p>Values are byte arrays with value semantics: a stored array is never
 * shared with the caller, so a mutation of an inserted or retrieved array does
 * not affect the store.
 *
 * <p>Port of the C++ {@code tagma_map::CoordMapN} string-key method set in
 * {@code sw/cpp/tagma_map/include/tagma_map/coord_map_n.h} and of the C++
 * {@code tagma_map::DynCoordMap} string-key method set in
 * {@code sw/cpp/tagma_map/include/tagma_map/dyn_coord_map.h}; the underlying
 * behavior mirrors the Rust {@code CoordMap} trait in
 * {@code sw/rust/map/src/coord_map.rs}.
 */
public interface CoordMap {

    /** The number of stored entries. */
    int len();

    /** Whether the store holds no entries. */
    default boolean isEmpty() {
        return len() == 0;
    }

    /** Removes all entries. */
    void clear();

    /**
     * Inserts a key-value pair.
     *
     * @return the previous value when the key already existed, mirroring the
     *         {@code HashMap::insert} contract
     * @throws IllegalArgumentException when the key length does not match the
     *         key length of the store, mirroring the Rust panic and the C++
     *         {@code std::invalid_argument}
     * @throws NullPointerException when {@code key} or {@code value} is null
     */
    Optional<byte[]> insert(String key, byte[] value);

    /**
     * The value stored under {@code key}.
     *
     * @return empty when the key is absent or its length does not match the key
     *         length of the store
     */
    Optional<byte[]> get(String key);

    /**
     * Removes a key-value pair.
     *
     * @return the removed value when present; empty when the key is absent or
     *         its length does not match the key length of the store
     */
    Optional<byte[]> remove(String key);

    /** Whether the store holds a value under {@code key}. */
    default boolean containsKey(String key) {
        return get(key).isPresent();
    }
}
