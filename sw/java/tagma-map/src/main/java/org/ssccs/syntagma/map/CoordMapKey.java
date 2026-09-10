package org.ssccs.syntagma.map;

import java.util.Optional;

/**
 * The {@link CoordKey} access of a fixed-size-key map, mirroring the Rust
 * {@code CoordMapKey} trait and the {@code _by_coordkey} members of the C++
 * method set.
 *
 * <p>The interface requires {@link CoordMap} and adds the {@code ByCoordKey}
 * members. {@link DynCoordMap} is a dynamic store and therefore implements
 * {@link CoordMap} only, exactly as in the references.
 *
 * <p>Port of the C++ {@code tagma_map::CoordMapN} coordinate-key method set in
 * {@code sw/cpp/tagma_map/include/tagma_map/coord_map_n.h}; the underlying
 * behavior mirrors the Rust {@code CoordMapKey} trait in
 * {@code sw/rust/map/src/coord_map.rs}.
 */
public interface CoordMapKey extends CoordMap {

    /**
     * Inserts a key-value pair under a {@link CoordKey}.
     *
     * @return the previous value when the key already existed
     * @throws IllegalArgumentException when the key length differs from the
     *         key length of the store
     * @throws NullPointerException when {@code key} or {@code value} is null
     */
    Optional<byte[]> insertByCoordKey(CoordKey key, byte[] value);

    /**
     * The value stored under a {@link CoordKey}.
     *
     * @return empty when the key is absent
     * @throws IllegalArgumentException when the key length differs from the
     *         key length of the store
     */
    Optional<byte[]> getByCoordKey(CoordKey key);

    /**
     * Removes a key-value pair by {@link CoordKey}.
     *
     * @return the removed value when present
     * @throws IllegalArgumentException when the key length differs from the
     *         key length of the store
     */
    Optional<byte[]> removeByCoordKey(CoordKey key);

    /** Whether the store holds a value under a {@link CoordKey}. */
    default boolean containsKeyByCoordKey(CoordKey key) {
        return getByCoordKey(key).isPresent();
    }
}
