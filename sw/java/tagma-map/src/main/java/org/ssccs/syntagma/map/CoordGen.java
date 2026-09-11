package org.ssccs.syntagma.map;

import java.util.List;
import java.util.Optional;
import java.util.OptionalInt;

import org.ssccs.syntagma.core.Coord;

/**
 * A strategy for converting a string key into a sequence of {@link Coord}
 * values, the fundamental mapping from application-level keys to Tagma's
 * coordinate space.
 *
 * <p>Two families exist. Dynamic strategies ({@link ByteWise},
 * {@link CharWise}) vary the path length with the key and are collision-free;
 * they must be backed by a depth-flexible store such as
 * {@code DynCoordSpace}. Static strategies ({@link Prefix}, {@link ByteFold})
 * fix the path length regardless of the key, which enables O(1) dense array
 * lookups at the cost of collisions from truncation or compression.
 *
 * <p>Java has no type aliases, so the Rust {@code type DefaultDynamic =
 * ByteWise} becomes {@link #DEFAULT_DYNAMIC}. The C++ free function
 * {@code string_to_coord_path} becomes the static
 * {@link #stringToCoordPath(String)}.
 *
 * <p>Port of the C++ {@code tagma_map::CoordGen} strategy set in
 * {@code sw/cpp/tagma_map/include/tagma_map/coord_gen.h}; the underlying
 * behavior mirrors the Rust {@code CoordGen} trait in
 * {@code sw/rust/map/src/coord_gen.rs}.
 */
public interface CoordGen {

    /** The default dynamic coordinate generation strategy, {@link ByteWise}. */
    CoordGen DEFAULT_DYNAMIC = ByteWise.INSTANCE;

    /** A human-readable strategy name, such as {@code "byte-wise"}. */
    String name();

    /**
     * Converts {@code key} into a list of coordinates owned by the caller.
     *
     * @return empty for the failures classified by {@link GenError}: an empty
     *         key, or text a strategy cannot decode
     * @throws NullPointerException when {@code key} is null
     */
    Optional<List<Coord>> generate(String key);

    /**
     * Whether the strategy guarantees an injective, collision-free mapping.
     * The dynamic strategies return {@code true}; the static strategies
     * {@link Prefix} and {@link ByteFold} return {@code false}.
     */
    boolean isInjective();

    /**
     * The fixed number of coordinates the strategy always produces, or empty
     * when the path length varies with the key.
     */
    OptionalInt fixedDepth();

    /**
     * The string-to-path conversion used by the dynamic store, delegating to
     * {@link ByteWise}. Mirrors the C++ free function
     * {@code string_to_coord_path} and the Rust
     * {@code tagma_map::string_to_coord_path}.
     *
     * @return empty for the empty string
     */
    static Optional<List<Coord>> stringToCoordPath(String key) {
        return ByteWise.INSTANCE.generate(key);
    }
}
