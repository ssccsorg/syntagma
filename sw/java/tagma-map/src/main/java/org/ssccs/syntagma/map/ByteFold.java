package org.ssccs.syntagma.map;

import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.List;
import java.util.Objects;
import java.util.Optional;
import java.util.OptionalInt;

import org.ssccs.syntagma.core.Coord;

/**
 * The static byte-fold strategy: XOR-folds all key bytes into {@code length}
 * accumulators, then maps each accumulator modulo 11172.
 *
 * <p>Accumulator {@code j} collects the bytes at positions {@code i} where
 * {@code i % length == j}. This is a lossy compression strategy, so multiple
 * keys can produce the same path. The path length always equals
 * {@code length}.
 *
 * <p>Java has no const generics, so the compile-time {@code N} of the C++
 * {@code ByteFold<N>} and the Rust {@code ByteFold<N>} becomes immutable
 * instance state validated by the constructor, where the references assert
 * {@code N > 0} at compile time.
 *
 * <p>Port of the C++ {@code tagma_map::ByteFold<N>} in
 * {@code sw/cpp/tagma_map/include/tagma_map/coord_gen.h}; the underlying
 * behavior mirrors the Rust {@code ByteFold<N>} in
 * {@code sw/rust/map/src/coord_gen.rs}.
 */
public final class ByteFold implements CoordGen {

    private final int length;

    /**
     * Creates a strategy that always produces {@code length} coordinates.
     *
     * @throws IllegalArgumentException when {@code length} is less than 1
     */
    public ByteFold(int length) {
        if (length < 1) {
            throw new IllegalArgumentException(
                    "ByteFold: length must be at least 1, got " + length);
        }
        this.length = length;
    }

    /** The fixed path length, the C++/Rust template parameter {@code N}. */
    public int length() {
        return length;
    }

    @Override
    public String name() {
        return "byte-fold";
    }

    @Override
    public Optional<List<Coord>> generate(String key) {
        Objects.requireNonNull(key, "key");
        if (key.isEmpty()) {
            return Optional.empty();
        }
        byte[] bytes = key.getBytes(StandardCharsets.UTF_8);
        int[] accumulators = new int[length];
        for (int i = 0; i < bytes.length; i++) {
            accumulators[i % length] ^= bytes[i] & 0xFF;
        }
        List<Coord> coords = new ArrayList<>(length);
        for (int value : accumulators) {
            coords.add(Coord.fromIndex(value % Coord.N_VALID).orElseThrow());
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
