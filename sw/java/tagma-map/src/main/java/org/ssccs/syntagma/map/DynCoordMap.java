package org.ssccs.syntagma.map;

import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;
import java.util.Map;
import java.util.Objects;
import java.util.Optional;

import org.ssccs.syntagma.core.Coord;
import org.ssccs.syntagma.core.CoordPath;
import org.ssccs.syntagma.core.DynCoordSpace;

/**
 * A hash-free, collision-free string map backed by {@link DynCoordSpace} with
 * byte-wise coordinate generation.
 *
 * <p>Any non-empty string is a valid key and lookup cost is
 * {@code O(key length)}; the path length equals the UTF-8 byte length of the
 * key. An empty key cannot produce a path and is rejected by every operation
 * without storing anything, mirroring the references, where the rejection is
 * reported with the same empty result as an absent key.
 *
 * <p>Port of the C++ {@code tagma_map::DynCoordMap} in
 * {@code sw/cpp/tagma_map/include/tagma_map/dyn_coord_map.h}; the underlying
 * behavior mirrors the Rust {@code DynCoordMap} in
 * {@code sw/rust/map/src/dyn_coord_map.rs}.
 */
public final class DynCoordMap implements CoordMap, CoordPathLookup {

    private final DynCoordSpace<byte[]> space = new DynCoordSpace<>();
    private int len;

    /** Creates an empty dynamic-mode store. */
    public DynCoordMap() {
    }

    @Override
    public Optional<byte[]> insert(String key, byte[] value) {
        Objects.requireNonNull(key, "key");
        Objects.requireNonNull(value, "value");
        Optional<List<Coord>> path = CoordGen.stringToCoordPath(key);
        if (path.isEmpty()) {
            return Optional.empty();
        }
        Optional<byte[]> previous = space.place(path.orElseThrow(), value.clone());
        if (previous.isEmpty()) {
            len += 1;
        }
        return previous;
    }

    @Override
    public Optional<byte[]> get(String key) {
        Objects.requireNonNull(key, "key");
        Optional<List<Coord>> path = CoordGen.stringToCoordPath(key);
        if (path.isEmpty()) {
            return Optional.empty();
        }
        return space.at(path.orElseThrow()).map(byte[]::clone);
    }

    @Override
    public Optional<byte[]> remove(String key) {
        Objects.requireNonNull(key, "key");
        Optional<List<Coord>> path = CoordGen.stringToCoordPath(key);
        if (path.isEmpty()) {
            return Optional.empty();
        }
        Optional<byte[]> removed = space.vacate(path.orElseThrow()).map(byte[]::clone);
        if (removed.isPresent()) {
            len -= 1;
        }
        return removed;
    }

    @Override
    public Optional<byte[]> getByCoordPath(CoordPath path) {
        Objects.requireNonNull(path, "path");
        return space.at(Arrays.asList(path.coords())).map(byte[]::clone);
    }

    /**
     * All {@code (key, value)} pairs in depth-first coordinate-ascending order.
     * Keys are reconstructed from the stored byte-wise path and decoded as
     * ISO-8859-1, so each character is the stored byte value of the references
     * ({@code std::string} and {@code Vec<u8>}). The references hand out
     * pointers into the store; Java hands out copies, so an iterated value is
     * never shared with the store.
     */
    public List<Map.Entry<String, byte[]>> iter() {
        List<Map.Entry<List<Coord>, byte[]>> entries = space.entries();
        List<Map.Entry<String, byte[]>> out = new ArrayList<>(entries.size());
        for (Map.Entry<List<Coord>, byte[]> entry : entries) {
            List<Coord> coords = entry.getKey();
            byte[] key = new byte[coords.size()];
            for (int i = 0; i < key.length; i++) {
                key[i] = (byte) coords.get(i).index();
            }
            out.add(Map.entry(new String(key, StandardCharsets.ISO_8859_1),
                    entry.getValue().clone()));
        }
        return out;
    }

    @Override
    public int len() {
        return len;
    }

    @Override
    public void clear() {
        space.clear();
        len = 0;
    }
}
