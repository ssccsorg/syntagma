package org.ssccs.syntagma.map;

import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.Objects;
import java.util.Optional;

import org.ssccs.syntagma.core.CoordPath;
import org.ssccs.syntagma.core.CoordSpaceN;

/**
 * A fixed-depth byte-key map over the {@link CoordSpaceN} tree, the sparse
 * store for any depth.
 *
 * <p>Keys are exactly {@code depth} UTF-8 bytes long, and lookup cost is
 * {@code O(depth)} tree traversal with no hashing and no collisions. The depth
 * is immutable instance state validated by the constructor, the Java form of
 * the C++ and Rust template parameter {@code N}.
 *
 * <p>The entry count is delegated to {@link CoordSpaceN#size()}, which counts
 * the same placement and removal events as the explicit counter of the
 * references.
 *
 * <p>The class is not final because {@link CoordMap2} fixes the depth at 2,
 * mirroring the C++ alias {@code using CoordMap2 = CoordMapN<2>}.
 *
 * <p>Port of the C++ {@code tagma_map::CoordMapN<N>} in
 * {@code sw/cpp/tagma_map/include/tagma_map/coord_map_n.h}; the underlying
 * behavior mirrors the Rust {@code CoordMapN<N>} in
 * {@code sw/rust/map/src/coord_map_n.rs}.
 */
public class CoordMapN implements CoordMapKey, CoordPathLookup {

    private final CoordSpaceN<byte[]> space;

    /**
     * Creates an empty store of {@code depth} levels.
     *
     * @param depth the exact key byte length, at least 1
     * @throws IllegalArgumentException when {@code depth} is less than 1
     */
    public CoordMapN(int depth) {
        this.space = new CoordSpaceN<>(depth);
    }

    // ------------------------------------------------------------------
    // String-key API
    // ------------------------------------------------------------------

    /**
     * {@inheritDoc}
     *
     * <p>The key must be exactly {@code depth} UTF-8 bytes long; a mismatch
     * throws instead of storing, mirroring the Rust panic and the C++
     * {@code std::invalid_argument}.
     */
    @Override
    public Optional<byte[]> insert(String key, byte[] value) {
        Objects.requireNonNull(key, "key");
        Objects.requireNonNull(value, "value");
        return insertByCoordKey(CoordKey.fromString(key, space.depth()), value);
    }

    /**
     * {@inheritDoc}
     *
     * <p>A key whose UTF-8 byte length is not {@code depth} is absent by
     * definition, mirroring the length guard of the references.
     */
    @Override
    public Optional<byte[]> get(String key) {
        Objects.requireNonNull(key, "key");
        byte[] bytes = key.getBytes(StandardCharsets.UTF_8);
        if (bytes.length != space.depth()) {
            return Optional.empty();
        }
        return getByCoordKey(new CoordKey(bytes));
    }

    /**
     * {@inheritDoc}
     *
     * <p>A key whose UTF-8 byte length is not {@code depth} is absent by
     * definition, mirroring the length guard of the references.
     */
    @Override
    public Optional<byte[]> remove(String key) {
        Objects.requireNonNull(key, "key");
        byte[] bytes = key.getBytes(StandardCharsets.UTF_8);
        if (bytes.length != space.depth()) {
            return Optional.empty();
        }
        return removeByCoordKey(new CoordKey(bytes));
    }

    // ------------------------------------------------------------------
    // CoordKey API
    // ------------------------------------------------------------------

    @Override
    public Optional<byte[]> insertByCoordKey(CoordKey key, byte[] value) {
        Objects.requireNonNull(key, "key");
        Objects.requireNonNull(value, "value");
        return space.placePath(key.toCoordPath(), value.clone());
    }

    @Override
    public Optional<byte[]> getByCoordKey(CoordKey key) {
        Objects.requireNonNull(key, "key");
        return space.atPath(key.toCoordPath()).map(byte[]::clone);
    }

    @Override
    public Optional<byte[]> removeByCoordKey(CoordKey key) {
        Objects.requireNonNull(key, "key");
        return space.vacatePath(key.toCoordPath()).map(byte[]::clone);
    }

    // ------------------------------------------------------------------
    // Path lookup and iteration
    // ------------------------------------------------------------------

    /**
     * {@inheritDoc}
     *
     * <p>A path whose length differs from the depth of the store is absent
     * rather than an error, which is what {@link CoordPathLookup} documents.
     * The generated paths of a {@link CoordCubeMap} query follow the query
     * geometry, so a center that cannot address this store yields no hits
     * instead of an exception raised from inside the query loop.
     * {@link CoordSpaceN} itself keeps the rejection for callers that address
     * it directly.
     */
    @Override
    public Optional<byte[]> getByCoordPath(CoordPath path) {
        Objects.requireNonNull(path, "path");
        if (path.length() != space.depth()) {
            return Optional.empty();
        }
        return space.atPath(path).map(byte[]::clone);
    }

    /**
     * All {@code (key, value)} pairs in depth-first coordinate-ascending order.
     * The references hand out pointers into the store; Java hands out copies,
     * so an iterated value is never shared with the store.
     */
    public List<Map.Entry<CoordKey, byte[]>> iter() {
        List<Map.Entry<CoordPath, byte[]>> entries = space.entries();
        List<Map.Entry<CoordKey, byte[]>> out = new ArrayList<>(entries.size());
        for (Map.Entry<CoordPath, byte[]> entry : entries) {
            out.add(Map.entry(CoordKey.fromCoordPath(entry.getKey()), entry.getValue().clone()));
        }
        return out;
    }

    // ------------------------------------------------------------------
    // Size and clearing
    // ------------------------------------------------------------------

    @Override
    public int len() {
        return space.size();
    }

    @Override
    public void clear() {
        space.clear();
    }
}
