package org.ssccs.syntagma.bench;

import java.util.ArrayList;
import java.util.List;

import org.ssccs.syntagma.core.Coord;
import org.ssccs.syntagma.core.CoordPath;

/**
 * The coordinate fixtures every scenario family shares: the {@code coord},
 * {@code paths_2d} and {@code paths_3d} free functions of
 * {@code sw/cpp/bench/bench.cpp}, which Java carries as a static facade
 * because the language has no free functions.
 *
 * <p>The generated path sets are the ones of the C++ helper functions: the
 * two-dimensional set walks {@code (i % N_VALID, (i / N_VALID) % N_VALID)} and
 * the three-dimensional set the same pair with a final coordinate of index
 * zero, so the first {@link Coord#N_VALID} entries of both sets are distinct
 * and the sets stay identical to the reference inputs.
 */
public final class BenchInputs {

    private BenchInputs() {
    }

    /**
     * The coordinate of {@code index}, the C++ {@code Coord::from_index(index).value()}.
     *
     * @throws IllegalArgumentException when {@code index} is outside
     *         {@code [0, Coord.N_VALID)}
     */
    public static Coord coord(int index) {
        return Coord.fromIndex(index).orElseThrow(() -> new IllegalArgumentException(
                "BenchInputs: coordinate index out of range [0, " + Coord.N_VALID + "): " + index));
    }

    /**
     * {@code count} paths of two coordinates, {@code (i % N_VALID, (i / N_VALID) % N_VALID)}.
     *
     * @throws IllegalArgumentException when {@code count} is negative
     */
    public static List<CoordPath> paths2d(int count) {
        requireCount(count);
        List<CoordPath> out = new ArrayList<>(count);
        for (int i = 0; i < count; i++) {
            out.add(CoordPath.fromArray(
                    coord(i % Coord.N_VALID), coord((i / Coord.N_VALID) % Coord.N_VALID)));
        }
        return out;
    }

    /**
     * {@code count} paths of three coordinates,
     * {@code (i % N_VALID, (i / N_VALID) % N_VALID, 0)}.
     *
     * @throws IllegalArgumentException when {@code count} is negative
     */
    public static List<CoordPath> paths3d(int count) {
        requireCount(count);
        List<CoordPath> out = new ArrayList<>(count);
        for (int i = 0; i < count; i++) {
            out.add(CoordPath.fromArray(coord(i % Coord.N_VALID),
                    coord((i / Coord.N_VALID) % Coord.N_VALID), coord(0)));
        }
        return out;
    }

    private static void requireCount(int count) {
        if (count < 0) {
            throw new IllegalArgumentException("BenchInputs: count must not be negative, got " + count);
        }
    }
}
