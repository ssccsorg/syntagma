package org.ssccs.syntagma.map;

import java.util.Optional;

import org.ssccs.syntagma.core.CoordPath;

/**
 * A path-keyed lookup over a coordinate map, the parameter type of the spatial
 * queries in {@link CoordCubeMap}.
 *
 * <p>The references make this surface a private helper trait bound to the
 * concrete map types, which fixes the path length at compile time. Java mirrors
 * the C++ {@code get_by_coord_path} member set instead: the interface is public
 * and its paths carry their own length, so the size of the map is known at
 * runtime through the path that is looked up.
 *
 * <p>Port of the C++ {@code get_by_coord_path} members in
 * {@code sw/cpp/tagma_map/include/tagma_map/coord_map_n.h} and
 * {@code sw/cpp/tagma_map/include/tagma_map/dyn_coord_map.h}; the underlying
 * behavior mirrors the Rust {@code CoordPathLookup} helper trait in
 * {@code sw/rust/map/src/coord_cube_map.rs}.
 */
public interface CoordPathLookup {

    /**
     * The value stored under a {@link CoordPath}.
     *
     * @return empty when the path is absent or its length does not match the
     *         path length of the store
     */
    Optional<byte[]> getByCoordPath(CoordPath path);
}
