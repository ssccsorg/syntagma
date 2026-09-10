package org.ssccs.syntagma.core;

import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.NoSuchElementException;
import java.util.Objects;
import java.util.Optional;
import java.util.OptionalInt;
import java.util.function.Supplier;
import java.util.function.UnaryOperator;

/**
 * A fixed-depth, hash-less, collision-free address table indexed by
 * {@link CoordPath}.
 *
 * <p>Each level is a direct array index over the 11,172 valid coordinates, so
 * navigation is closed-form: one array access per level, with no hashing and no
 * collision resolution at any depth. Levels allocate lazily; only paths that
 * are actually placed consume nodes.
 *
 * <p>Java has no const generics, so the C++/Rust compile-time depth tag
 * ({@code CoordSpaceN<N, V>}) becomes immutable instance state validated by the
 * constructor. The C++ aliases {@code CoordSpaceN1}, {@code CoordSpaceN2},
 * {@code CoordSpaceN3}, {@code CoordSpaceN6}, {@code CoordSpaceN12} and
 * {@code CoordSpaceN19} correspond to depth arguments 1, 2, 3, 6, 12 and 19.
 *
 * <p>The single-coordinate member set ({@code at}, {@code atMut},
 * {@code occupied}, {@code place}, {@code vacate}, {@code entry}) mirrors the
 * C++ members that {@code std::enable_if} restricts to {@code N == 1}. Because
 * the depth is runtime state in Java, those calls are accepted by the compiler
 * for every instance and throw {@link UnsupportedOperationException} when the
 * depth is not 1.
 *
 * <p>{@link #equals(Object)} and {@link #hashCode()} mirror the Rust
 * {@code PartialEq}, which compares the depth, the length and the tree
 * structure. Nodes are allocated lazily and never pruned, so two spaces that
 * hold identical entries compare unequal once one of them has allocated a node
 * whose value was later vacated. Porting {@code equals} as a container
 * comparison of the entries would diverge from the Rust reference.
 * {@link #clear()} follows the C++ port instead and discards the tree, so a
 * cleared space compares equal to a fresh one.
 *
 * <p>Port of the C++ {@code tagma::CoordSpaceN<N, V>} in
 * {@code sw/cpp/tagma_core/include/tagma_core/coord_space_n.h}; the underlying
 * behavior mirrors the Rust {@code CoordSpaceN<N, V>} in
 * {@code sw/rust/core/src/coord_space_n.rs}.
 *
 * @param <V> the stored value type
 */
public final class CoordSpaceN<V> {

    /** The number of slots per level, always 11,172. */
    public static final int CAPACITY = Coord.N_VALID;

    private final int depth;
    private Node root;
    private int size;

    /**
     * Creates an empty space of {@code depth} levels.
     *
     * @throws IllegalArgumentException when {@code depth} is less than 1
     */
    public CoordSpaceN(int depth) {
        if (depth < 1) {
            throw new IllegalArgumentException("CoordSpaceN depth must be at least 1, got " + depth);
        }
        this.depth = depth;
        this.root = new Node(depth == 1);
    }

    /** The number of tree levels, the C++/Rust {@code N} template argument. */
    public int depth() {
        return depth;
    }

    /** The number of entries. */
    public int size() {
        return size;
    }

    /** Whether the space holds no entries. */
    public boolean isEmpty() {
        return size == 0;
    }

    /**
     * The maximum number of entries: 11,172 for depth 1; empty for deeper
     * spaces, whose tree grows dynamically. Mirrors the C++
     * {@code std::optional<std::size_t>} and the Rust {@code Option<usize>}.
     */
    public OptionalInt capacity() {
        return depth == 1 ? OptionalInt.of(CAPACITY) : OptionalInt.empty();
    }

    // ------------------------------------------------------------------
    // Path access
    // ------------------------------------------------------------------

    /**
     * The value at {@code path}.
     *
     * @return empty when the path holds no value
     * @throws IllegalArgumentException when {@code path.length() != depth()}
     */
    public Optional<V> atPath(CoordPath path) {
        Coord[] coords = coordsOf(path);
        Node leaf = descend(coords, false);
        if (leaf == null) {
            return Optional.empty();
        }
        return Optional.ofNullable(slotValue(leaf.entries[coords[depth - 1].index()]));
    }

    /**
     * A mutable handle to the value at {@code path}.
     *
     * @return empty when the path holds no value, mirroring the C++
     *         {@code at_path_mut} pointer return
     * @throws IllegalArgumentException when {@code path.length() != depth()}
     */
    public Optional<ValueRef<V>> atPathMut(CoordPath path) {
        Coord[] coords = coordsOf(path);
        Node leaf = descend(coords, false);
        if (leaf == null || leaf.entries[coords[depth - 1].index()] == null) {
            return Optional.empty();
        }
        return Optional.of(new ValueRef<>(this, path));
    }

    /**
     * Places {@code value} at {@code path}, creating missing levels.
     *
     * @return the previous value when the path was occupied
     * @throws IllegalArgumentException when {@code path.length() != depth()}
     * @throws NullPointerException when {@code value} is null
     */
    public Optional<V> placePath(CoordPath path, V value) {
        Objects.requireNonNull(value, "null value");
        Coord[] coords = coordsOf(path);
        Node leaf = descend(coords, true);
        int index = coords[depth - 1].index();
        Object previous = leaf.entries[index];
        leaf.entries[index] = value;
        if (previous == null) {
            size += 1;
        }
        return Optional.ofNullable(slotValue(previous));
    }

    /**
     * Removes the value at {@code path}.
     *
     * @return the removed value when the path was occupied
     * @throws IllegalArgumentException when {@code path.length() != depth()}
     */
    public Optional<V> vacatePath(CoordPath path) {
        Coord[] coords = coordsOf(path);
        Node leaf = descend(coords, false);
        if (leaf == null) {
            return Optional.empty();
        }
        int index = coords[depth - 1].index();
        Object previous = leaf.entries[index];
        leaf.entries[index] = null;
        if (previous != null) {
            size -= 1;
        }
        return Optional.ofNullable(slotValue(previous));
    }

    /**
     * Removes all entries. Mirrors the C++ port, which discards the allocated
     * tree; the Rust reference clears the nodes in place. The difference is
     * observable only through the structural {@link #equals(Object)}: a
     * cleared Java space compares equal to a fresh one, while a cleared Rust
     * space retains its nodes.
     */
    public void clear() {
        root = new Node(depth == 1);
        size = 0;
    }

    // ------------------------------------------------------------------
    // Single-coordinate access (depth 1)
    // ------------------------------------------------------------------

    /**
     * The value at {@code coord}, for depth-1 spaces.
     *
     * @throws UnsupportedOperationException when {@link #depth()} is not 1
     */
    public Optional<V> at(Coord coord) {
        requireDepthOne("at");
        return Optional.ofNullable(slotValue(root.entries[coord.index()]));
    }

    /**
     * A mutable handle to the value at {@code coord}, for depth-1 spaces.
     *
     * @return empty when the slot is vacant
     * @throws UnsupportedOperationException when {@link #depth()} is not 1
     */
    public Optional<ValueRef<V>> atMut(Coord coord) {
        requireDepthOne("atMut");
        if (root.entries[coord.index()] == null) {
            return Optional.empty();
        }
        return Optional.of(new ValueRef<>(this, CoordPath.fromArray(coord)));
    }

    /**
     * Whether the depth-1 slot for {@code coord} holds a value.
     *
     * @throws UnsupportedOperationException when {@link #depth()} is not 1
     */
    public boolean occupied(Coord coord) {
        requireDepthOne("occupied");
        return root.entries[coord.index()] != null;
    }

    /**
     * Places {@code value} at {@code coord}, for depth-1 spaces.
     *
     * @return the previous value when the slot was occupied
     * @throws UnsupportedOperationException when {@link #depth()} is not 1
     * @throws NullPointerException when {@code value} is null
     */
    public Optional<V> place(Coord coord, V value) {
        requireDepthOne("place");
        Objects.requireNonNull(value, "null value");
        Object previous = root.entries[coord.index()];
        root.entries[coord.index()] = value;
        if (previous == null) {
            size += 1;
        }
        return Optional.ofNullable(slotValue(previous));
    }

    /**
     * Removes the value at {@code coord}, for depth-1 spaces.
     *
     * @return the removed value when the slot was occupied
     * @throws UnsupportedOperationException when {@link #depth()} is not 1
     */
    public Optional<V> vacate(Coord coord) {
        requireDepthOne("vacate");
        Object previous = root.entries[coord.index()];
        root.entries[coord.index()] = null;
        if (previous != null) {
            size -= 1;
        }
        return Optional.ofNullable(slotValue(previous));
    }

    /**
     * Returns the entry handle for {@code coord}, for depth-1 spaces.
     *
     * @throws UnsupportedOperationException when {@link #depth()} is not 1
     */
    public Entry<V> entry(Coord coord) {
        requireDepthOne("entry");
        return new Entry<>(this, coord);
    }

    // ------------------------------------------------------------------
    // Iteration
    // ------------------------------------------------------------------

    /**
     * All present paths in depth-first coordinate-ascending order. The Rust
     * reference yields a lazy iterator; the Java port materializes it eagerly,
     * as the C++ port does.
     */
    public List<CoordPath> paths() {
        List<CoordPath> out = new ArrayList<>(size);
        collectPaths(root, new Coord[depth], 0, out);
        return out;
    }

    /**
     * All {@code (path, value)} pairs in depth-first coordinate-ascending
     * order, as a snapshot.
     */
    public List<Map.Entry<CoordPath, V>> entries() {
        List<Map.Entry<CoordPath, V>> out = new ArrayList<>(size);
        collectEntries(root, new Coord[depth], 0, out);
        return out;
    }

    /**
     * All {@code (path, value)} pairs below {@code prefix} in depth-first
     * coordinate-ascending order. Mirrors the Rust {@code iter_prefix}, with
     * the same eager representation as {@link #entries()}.
     *
     * @return empty when {@code prefix.size() >= depth()} or when the prefix
     *         path does not exist in the tree; a present, possibly empty, list
     *         otherwise
     */
    public Optional<List<Map.Entry<CoordPath, V>>> entriesPrefix(List<Coord> prefix) {
        Objects.requireNonNull(prefix, "prefix");
        int level = prefix.size();
        if (level >= depth) {
            return Optional.empty();
        }
        Coord[] base = new Coord[depth];
        Node node = root;
        for (int i = 0; i < level; i++) {
            Coord coord = Objects.requireNonNull(prefix.get(i), "null coordinate in prefix");
            base[i] = coord;
            Object child = node.entries[coord.index()];
            if (child == null) {
                return Optional.empty();
            }
            node = (Node) child;
        }
        List<Map.Entry<CoordPath, V>> out = new ArrayList<>();
        collectEntries(node, base, level, out);
        return Optional.of(out);
    }

    /**
     * Returns an independent copy of the tree, mirroring the Rust
     * {@code Clone} derive that the C++ port omits. Values are shared, not
     * cloned: Java has no generic clone contract, and the port treats stored
     * values as immutable.
     */
    public CoordSpaceN<V> copy() {
        CoordSpaceN<V> copy = new CoordSpaceN<>(depth);
        copy.root = copyNode(root);
        copy.size = size;
        return copy;
    }

    // ------------------------------------------------------------------
    // Value semantics
    // ------------------------------------------------------------------

    /**
     * Structural equality, mirroring the Rust {@code PartialEq}: same depth,
     * same length and structurally equal trees.
     */
    @Override
    public boolean equals(Object other) {
        if (this == other) {
            return true;
        }
        if (!(other instanceof CoordSpaceN<?> space)) {
            return false;
        }
        return depth == space.depth && size == space.size && nodeEquals(root, space.root);
    }

    /** Hash consistent with {@link #equals(Object)}; the Rust reference declares no hash. */
    @Override
    public int hashCode() {
        return 31 * (31 * Integer.hashCode(depth) + Integer.hashCode(size)) + nodeHash(root);
    }

    /**
     * Displays the space, mirroring the Rust {@code Debug} impl: the depth, the
     * length and the root node role with its occupied slot count.
     */
    @Override
    public String toString() {
        return "CoordSpace { N: " + depth + ", len: " + size + ", root: " + nodeDebug(root) + " }";
    }

    // ------------------------------------------------------------------
    // Internals
    // ------------------------------------------------------------------

    /**
     * One tree node, mirroring the C++ {@code SpaceNode} pair: a leaf holds
     * values, a branch holds child nodes. The C++ port splits the two roles
     * into {@code SpaceNode<0, V>} and {@code SpaceNode<D, V>}; Java carries
     * the role as a flag because the depth is runtime state.
     */
    private static final class Node {

        /** Values in a leaf, child nodes in a branch. */
        private final Object[] entries = new Object[CAPACITY];
        private final boolean leaf;

        private Node(boolean leaf) {
            this.leaf = leaf;
        }
    }

    private Coord[] coordsOf(CoordPath path) {
        Objects.requireNonNull(path, "path");
        if (path.length() != depth) {
            throw new IllegalArgumentException(
                    "CoordSpaceN path length " + path.length() + " does not match depth " + depth);
        }
        return path.coords();
    }

    private void requireDepthOne(String method) {
        if (depth != 1) {
            throw new UnsupportedOperationException(
                    "CoordSpaceN::" + method + " requires depth 1, got depth " + depth);
        }
    }

    /**
     * Walks the branch levels to the leaf holding {@code coords[depth - 1]},
     * creating missing levels when {@code create} is set and returning null
     * otherwise.
     */
    private Node descend(Coord[] coords, boolean create) {
        Node node = root;
        for (int level = 0; level < depth - 1; level++) {
            int index = coords[level].index();
            Object child = node.entries[index];
            if (child == null) {
                if (!create) {
                    return null;
                }
                child = new Node(level == depth - 2);
                node.entries[index] = child;
            }
            node = (Node) child;
        }
        return node;
    }

    private static Node copyNode(Node node) {
        Node copy = new Node(node.leaf);
        for (int index = 0; index < CAPACITY; index++) {
            Object entry = node.entries[index];
            if (entry != null) {
                copy.entries[index] = node.leaf ? entry : copyNode((Node) entry);
            }
        }
        return copy;
    }

    private static void collectPaths(Node node, Coord[] prefix, int level, List<CoordPath> out) {
        for (int index = 0; index < CAPACITY; index++) {
            Object entry = node.entries[index];
            if (entry == null) {
                continue;
            }
            prefix[level] = Coord.raw(index);
            if (node.leaf) {
                out.add(CoordPath.fromArray(prefix));
            } else {
                collectPaths((Node) entry, prefix, level + 1, out);
            }
        }
    }

    private void collectEntries(
            Node node, Coord[] prefix, int level, List<Map.Entry<CoordPath, V>> out) {
        for (int index = 0; index < CAPACITY; index++) {
            Object entry = node.entries[index];
            if (entry == null) {
                continue;
            }
            prefix[level] = Coord.raw(index);
            if (node.leaf) {
                out.add(Map.entry(CoordPath.fromArray(prefix), slotValue(entry)));
            } else {
                collectEntries((Node) entry, prefix, level + 1, out);
            }
        }
    }

    private static boolean nodeEquals(Node a, Node b) {
        if (a.leaf != b.leaf) {
            return false;
        }
        for (int index = 0; index < CAPACITY; index++) {
            Object left = a.entries[index];
            Object right = b.entries[index];
            if (left == null || right == null) {
                if (left != right) {
                    return false;
                }
            } else if (a.leaf ? !left.equals(right) : !nodeEquals((Node) left, (Node) right)) {
                return false;
            }
        }
        return true;
    }

    private static int nodeHash(Node node) {
        int hash = Boolean.hashCode(node.leaf);
        for (Object entry : node.entries) {
            if (entry != null) {
                hash = 31 * hash + (node.leaf ? entry.hashCode() : nodeHash((Node) entry));
            }
        }
        return hash;
    }

    private static String nodeDebug(Node node) {
        int occupied = 0;
        for (Object entry : node.entries) {
            if (entry != null) {
                occupied += 1;
            }
        }
        return node.leaf
                ? "Leaf { occupied: " + occupied + " }"
                : "Branch { children: " + occupied + " }";
    }

    /**
     * Reads a leaf slot. One suppression covers the class: Java generics are
     * erased, so the element type of a slot array cannot be reified and every
     * value read is a checked cast at the call site that stores it.
     */
    @SuppressWarnings("unchecked")
    private V slotValue(Object slot) {
        return (V) slot;
    }

    // ------------------------------------------------------------------
    // Nested handles
    // ------------------------------------------------------------------

    /**
     * A mutable reference to a value inside a {@link CoordSpaceN}, the Java
     * counterpart of the C++ {@code V*} / Rust {@code &mut V} write access.
     *
     * @param <V> the stored value type
     */
    public static final class ValueRef<V> {

        private final CoordSpaceN<V> space;
        private final CoordPath path;

        private ValueRef(CoordSpaceN<V> space, CoordPath path) {
            this.space = space;
            this.path = path;
        }

        /** The referenced path. */
        public CoordPath path() {
            return path;
        }

        /** The current value; throws when the entry has been vacated. */
        public V get() {
            return space.atPath(path).orElseThrow(
                    () -> new NoSuchElementException("CoordSpaceN entry vacant: " + path));
        }

        /** Replaces the stored value; throws when the entry has been vacated. */
        public void set(V value) {
            Objects.requireNonNull(value, "null value");
            if (space.atPath(path).isEmpty()) {
                throw new NoSuchElementException("CoordSpaceN entry vacant: " + path);
            }
            space.placePath(path, value);
        }

        /** Applies {@code updater} to the stored value and writes it back. */
        public V update(UnaryOperator<V> updater) {
            V updated = Objects.requireNonNull(updater.apply(get()), "updater returned null");
            set(updated);
            return updated;
        }
    }

    /**
     * An entry handle for a depth-1 slot, mirroring the C++ inner {@code Entry}
     * and the Rust {@code Entry} for {@code CoordSpaceN<1, V>}.
     *
     * @param <V> the stored value type
     */
    public static final class Entry<V> {

        private final CoordSpaceN<V> space;
        private final Coord coord;

        private Entry(CoordSpaceN<V> space, Coord coord) {
            this.space = space;
            this.coord = coord;
        }

        /** The coordinate this entry addresses. */
        public Coord coord() {
            return coord;
        }

        /**
         * Returns a mutable handle to the value at this entry's coordinate,
         * inserting {@code value} when the slot is vacant. Never overwrites an
         * existing value.
         */
        public ValueRef<V> orInsert(V value) {
            Objects.requireNonNull(value, "null value");
            if (!space.occupied(coord)) {
                space.place(coord, value);
            }
            return new ValueRef<>(space, CoordPath.fromArray(coord));
        }

        /** Like {@link #orInsert(Object)} but computes the value lazily. */
        public ValueRef<V> orInsertWith(Supplier<V> factory) {
            Objects.requireNonNull(factory, "factory");
            if (!space.occupied(coord)) {
                space.place(coord, Objects.requireNonNull(factory.get(), "factory returned null"));
            }
            return new ValueRef<>(space, CoordPath.fromArray(coord));
        }

        /**
         * Applies {@code updater} to the stored value when the slot is
         * occupied, mirroring the Rust {@code Entry::and_modify} that the C++
         * {@code Entry} omits. Returns this entry, so the call chains with
         * {@link #orInsert(Object)}.
         */
        public Entry<V> andModify(UnaryOperator<V> updater) {
            Objects.requireNonNull(updater, "updater");
            space.at(coord).ifPresent(current ->
                    space.place(coord, Objects.requireNonNull(updater.apply(current), "updater returned null")));
            return this;
        }
    }
}
