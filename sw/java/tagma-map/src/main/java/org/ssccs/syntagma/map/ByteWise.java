package org.ssccs.syntagma.map;

import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.List;
import java.util.Objects;
import java.util.Optional;
import java.util.OptionalInt;

import org.ssccs.syntagma.core.Coord;

/**
 * The byte-wise dynamic strategy: each UTF-8 byte maps to one coordinate.
 *
 * <p>Byte values {@code 0..255} are always inside the valid coordinate range
 * {@code 0..11171}, so the mapping is injective and collision-free. The path
 * length equals the UTF-8 byte length of the key, which is the key byte length
 * of the references.
 *
 * <p>A Java string that is not well-formed UTF-16, meaning an unpaired
 * surrogate, has no counterpart in the references and is encoded with the
 * replacement byte of the platform encoder. {@link CharWise} rejects such a
 * string instead, because a surrogate is not a Unicode scalar value and cannot
 * be split into the coordinate pair that strategy produces.
 *
 * <p>This is the default strategy behind {@link CoordGen#DEFAULT_DYNAMIC},
 * {@link CoordGen#stringToCoordPath(String)} and {@link DynCoordMap}.
 *
 * <p>Port of the C++ {@code tagma_map::ByteWise} in
 * {@code sw/cpp/tagma_map/include/tagma_map/coord_gen.h}; the underlying
 * behavior mirrors the Rust {@code ByteWise} in
 * {@code sw/rust/map/src/coord_gen.rs}.
 */
public final class ByteWise implements CoordGen {

    /** The shared stateless instance, the Java counterpart of the unit struct. */
    public static final ByteWise INSTANCE = new ByteWise();

    private ByteWise() {
    }

    @Override
    public String name() {
        return "byte-wise";
    }

    @Override
    public Optional<List<Coord>> generate(String key) {
        Objects.requireNonNull(key, "key");
        if (key.isEmpty()) {
            return Optional.empty();
        }
        byte[] bytes = key.getBytes(StandardCharsets.UTF_8);
        List<Coord> coords = new ArrayList<>(bytes.length);
        for (byte value : bytes) {
            coords.add(Coord.fromIndex(value & 0xFF).orElseThrow());
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
