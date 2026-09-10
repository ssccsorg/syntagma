package org.ssccs.syntagma.map;

import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.List;
import java.util.Objects;
import java.util.Optional;
import java.util.OptionalInt;

import org.ssccs.syntagma.core.Coord;

/**
 * The static prefix strategy: the first {@code length} bytes of the key map to
 * that many coordinates, and shorter keys are zero-padded.
 *
 * <p>This is a lossy truncation strategy, so two keys sharing the first
 * {@code length} bytes collide. The path length always equals {@code length}.
 *
 * <p>Java has no const generics, so the compile-time {@code N} of the C++
 * {@code Prefix<N>} and the Rust {@code Prefix<N>} becomes immutable instance
 * state validated by the constructor, where the references assert
 * {@code N > 0} at compile time.
 *
 * <p>Port of the C++ {@code tagma_map::Prefix<N>} in
 * {@code sw/cpp/tagma_map/include/tagma_map/coord_gen.h}; the underlying
 * behavior mirrors the Rust {@code Prefix<N>} in
 * {@code sw/rust/map/src/coord_gen.rs}.
 */
public final class Prefix implements CoordGen {

    private final int length;

    /**
     * Creates a strategy that always produces {@code length} coordinates.
     *
     * @throws IllegalArgumentException when {@code length} is less than 1
     */
    public Prefix(int length) {
        if (length < 1) {
            throw new IllegalArgumentException(
                    "Prefix: length must be at least 1, got " + length);
        }
        this.length = length;
    }

    /** The fixed path length, the C++/Rust template parameter {@code N}. */
    public int length() {
        return length;
    }

    @Override
    public String name() {
        return "prefix";
    }

    @Override
    public Optional<List<Coord>> generate(String key) {
        Objects.requireNonNull(key, "key");
        if (key.isEmpty()) {
            return Optional.empty();
        }
        byte[] bytes = key.getBytes(StandardCharsets.UTF_8);
        List<Coord> coords = new ArrayList<>(length);
        for (int i = 0; i < length; i++) {
            int value = i < bytes.length ? bytes[i] & 0xFF : 0;
            coords.add(Coord.fromIndex(value).orElseThrow());
        }
        return Optional.of(coords);
    }

    @Override
    public boolean isInjective() {
        return false;
    }

    @Override
    public OptionalInt fixedDepth() {
        return OptionalInt.of(length);
    }
}
