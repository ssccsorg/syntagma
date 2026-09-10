package org.ssccs.syntagma.map;

import java.util.ArrayList;
import java.util.List;
import java.util.Objects;
import java.util.Optional;
import java.util.OptionalInt;

import org.ssccs.syntagma.core.Coord;

/**
 * The char-wise dynamic strategy: each Unicode scalar value maps to two
 * coordinates via {@code c0 = codePoint / 11172} and
 * {@code c1 = codePoint % 11172}.
 *
 * <p>Since {@code 11172 * 100} exceeds the maximum Unicode scalar value
 * {@code 0x10FFFF}, every scalar produces a unique pair, so the mapping is
 * injective and collision-free. The path length is twice the number of scalar
 * values.
 *
 * <p>The references decode UTF-8 and reject malformed text. A Java string is
 * UTF-16, whose only malformed state is an unpaired surrogate; such a string is
 * rejected here as well, keeping the same failure shape as the invalid UTF-8 of
 * the references.
 *
 * <p>Port of the C++ {@code tagma_map::CharWise} in
 * {@code sw/cpp/tagma_map/include/tagma_map/coord_gen.h}; the underlying
 * behavior mirrors the Rust {@code CharWise} in
 * {@code sw/rust/map/src/coord_gen.rs}.
 */
public final class CharWise implements CoordGen {

    /** The shared stateless instance, the Java counterpart of the unit struct. */
    public static final CharWise INSTANCE = new CharWise();

    private CharWise() {
    }

    @Override
    public String name() {
        return "char-wise";
    }

    @Override
    public Optional<List<Coord>> generate(String key) {
        Objects.requireNonNull(key, "key");
        if (key.isEmpty()) {
            return Optional.empty();
        }
        int[] codePoints = key.codePoints().toArray();
        List<Coord> coords = new ArrayList<>(codePoints.length * 2);
        for (int codePoint : codePoints) {
            if (codePoint >= Character.MIN_SURROGATE && codePoint <= Character.MAX_SURROGATE) {
                // An unpaired surrogate is not a Unicode scalar value; the
                // references reject the equivalent malformed UTF-8.
                return Optional.empty();
            }
            coords.add(Coord.fromIndex(codePoint / Coord.N_VALID).orElseThrow());
            coords.add(Coord.fromIndex(codePoint % Coord.N_VALID).orElseThrow());
        }
        return Optional.of(coords);
    }

    @Override
    public boolean isInjective() {
        return true;
    }

    @Override
    public OptionalInt fixedDepth() {
        return OptionalInt.empty();
    }
}
