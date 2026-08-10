# From Map to Space: the day Tagma shed its HashMap skin

The code still compiled. Every test passed. But the naming told a lie.

## The trap

Tagma started as `CoordMap`. Not maliciously — it felt natural. A Coord was the key, and the value was the value. HashMap does the same thing, so why not mirror its API? `insert`, `get`, `remove`, `contains_key`, `keys()` — all the familiar friends. The user would feel at home.

The trap was that every one of those names carried HashMap's ontology into Tagma's codebase. A `Map` has keys. A `Map` stores entries. A `Map` answers `get(key)`. When you think in maps, you think in lossy compression: destroy the key's structure, store only the mapping.

Tagma is the opposite. It preserves every bit of structure. The coordinate is the address, the address carries its own axis decomposition, and queries are arithmetic on those axes. A map cannot express "find everything near this coordinate" without being turned inside out. A space answers it natively.

## The rename cascade

The surface change was simple — rename types and methods. But each rename exposed a deeper conceptual debt:

| Rename | What it fixed |
|--------|--------------|
| `CoordMap` → `CoordSpace` | "Map" implies key-value compression. "Space" implies uncompressed positional structure. |
| `CoordTreeMap` → `CoordSpaceN` | The tree is an implementation detail, not the concept. |
| `DynCoordMap` → `DynCoordSpace` | Same — the "Map" suffix was a category error. |
| `flat.rs` / `map.rs` / `dyn_coord.rs` → `coord_space.rs` / `coord_space_n.rs` / `dyn_coord_space.rs` | File names are API. They should say what the types are, not what data structure backs them. |
| `contains_key` → `occupied` | There is no key. A coordinate is either occupied or vacant. |
| `keys()` → `coords()` | It was returning Coords all along. The name `keys()` was a lie. |
| `get_key_value` → `get_entry` | Same lie, different method. |
| `Entry::key()` → `Entry::coord()` | The entry is at a coordinate in space, not at a key in a map. |
| `insert` → `place` | You place a value at a coordinate in space. You do not insert into space. |
| `get` → `at` | You look at a coordinate to see what is there. |
| `remove` → `vacate` | You vacate a position in space. |

The mechanical part — sed replace, rebuild, fix errors — took under an hour. The conceptual part took days of argument and a white paper revision.

## The API that survived (and why)

`insert`/`get`/`remove` were the hardest to let go of. Not because they were right, but because they are universal. Vec has `insert`. BTreeMap has `get`. These are not HashMap's words; they are collection I/O's words.

But they ARE HashMap's frame. When you name a method `insert`, the user reaches for HashMap's semantics: hashing, collisions, rehashing. Tagma has none of those. The cost of `place`/`at`/`vacate` is one unfamiliar minute; the cost of `insert`/`get`/`remove` is permanent misdirection.

`entry()` survived the cut. Entry API is not HashMap-specific — it is a general pattern for "get or create" that exists across collections. The internal types are still called `FlatEntry`/`OccupiedEntry`/`VacantEntry`, which is honest: "occupied" and "vacant" are spatial concepts.

## The benchmark that proved it

The axis filter benchmark compares CoordSpace and HashMap on the same workload: scan all 11,172 entries, decompose each coordinate into its three axes, count those where medial == 10. Same algorithm, same data, same filter logic. The only difference is memory layout.

```
Spatial/axis_filter_medial_10/CoordSpace    513 ns    1.04 Gelem/s
Spatial/axis_filter_medial_10/HashMap     21,357 ns   24.9 Melem/s
```

41x. Not because CoordSpace has a better filter algorithm — it doesn't. Because CoordSpace stores values in a contiguous array that the CPU prefetcher can walk in its sleep, while HashMap scatters them across fragmented buckets that stall on every cache line.

This is the empirical proof that "map is lossy compression, space is uncompressed original." The compression (hashing) destroys the spatial locality that the CPU depends on. CoordSpace preserves it. The benchmark is not about Tagma being faster at HashMap's job. It is about Tagma doing a job HashMap cannot attempt at competitive speed.

## What did not change

CoordSet kept `insert`/`contains`/`remove`. A set is a set. There is no spatial metaphor to enforce — a set is either a member or not. The HashMap pattern was not harmful here because CoordSet never pretended to be a map.

The internal entry struct fields (`map: &'a mut CoordSpace`) were renamed to `space`. This was a private rename that affected no API consumers, but it closed the last gap between what the code said and what it meant.

## What remains

The white paper (`wp.qmd`) and README were updated to reflect the new naming. The benchmark labels changed from `TagmaMap/...` to `CoordSpace/...`. The test file split into `no_alloc.rs` (17 tests, no heap) and `coord_space.rs` (18 tests, alloc-dependent) so that `--no-default-features` correctly exercises the embedded target path.

The thing that cannot be renamed is the history. The git log still says "CoordMap", "insert", "get". Future readers will see the old names and wonder. A devlog is the best we can do to mark the transition.
