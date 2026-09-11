package org.ssccs.syntagma.core;

import java.nio.ByteBuffer;
import java.util.Arrays;
import java.util.Objects;
import java.util.Optional;
import java.util.function.BiPredicate;

/**
 * A dense, anonymous, lazily committed direct-address space of depth 3: the
 * Java counterpart of the mmap-backed C++ {@code tagma::CoordSpaceM<N, V>}.
 *
 * <p>Addressing is direct: a {@link CoordPath} maps to a linear index by the
 * base-11172 fold, so placement and lookup are a single slot access, with no
 * hashing and no collisions. The slot count is fixed at 11,172^3
 * (1,394,417,360,448) and the constructor rejects every other depth.
 *
 * <p>Anonymous memory, no persistence and no format. The C++ and Rust
 * references reserve the slot region with an anonymous {@code mmap}
 * ({@code MAP_NORESERVE} on Linux), so the kernel commits pages on first write
 * and no byte of the space ever reaches a file. Java has no portable anonymous
 * mapping, so the port materializes its windows from off-heap direct memory
 * with {@link ByteBuffer#allocateDirect} on first touch. The operating system
 * hands back zero-filled memory, which keeps the reference slot invariant that
 * a zero-filled region reads as vacant. The space owns no persistence and no
 * on-disk format: coordinates stay in-memory values here, and materializing
 * them into pages, layouts or files belongs to chton.
 *
 * <p>Windows. A {@code ByteBuffer} is limited to {@code Integer.MAX_VALUE}
 * bytes, so the slot region is materialized window by window, each window sized
 * to the largest multiple of the slot stride that fits. A placement allocates
 * the window that contains it; reads never allocate, so an untouched region
 * costs no memory and reads as vacant. A placement materializes its whole
 * window, so a write pattern whose index span crosses windows pays one window
 * per crossed window, where the native references reserve the entire region in
 * a single mapping and commit only the touched pages. That coarser commit
 * granularity is the semantic difference of this port; the first placement in a
 * fresh space allocates one window of up to {@code Integer.MAX_VALUE} bytes.
 *
 * <p>Accounting and release. Direct buffers are accounted against
 * {@code -XX:MaxDirectMemorySize}, whose default is derived from the maximum
 * heap. The space needs at least one window before it can hold a value, about
 * 2 GiB for the 32-bit value mapping, so the budget has to exceed one window
 * before the first placement. The build pins the floor at
 * {@code -XX:MaxDirectMemorySize=3g} for the test and benchmark JVMs of
 * {@code sw/java}, and {@code sw/java/run.sh} exports the same value for
 * {@code --bench}, because the JVM default depends on the runner's heap size;
 * a caller running outside the build has to set the value explicitly, and a
 * consumer that places values across many windows has to raise it further. A
 * placement that cannot reserve its window raises {@link OutOfMemoryError}.
 * Java cannot release off-heap memory deterministically before the FFM API, so
 * {@link #close()} drops the window references and the buffers become
 * unreachable: their memory returns to the operating system when the
 * {@code Cleaner} that the buffer implementation registers runs, typically at
 * the next garbage collection. {@link #clear()} drops them the same way, so a
 * cleared space reads as freshly zero-filled memory again.
 *
 * <p>The C++ port omits the Rust {@code Clone} and {@code PartialEq}, because a
 * byte-wise copy or comparison would walk the multi-terabyte region page by
 * page. The Java port keeps the same surface: no copy method, identity
 * equality.
 *
 * <p>Slot encoding: one presence byte followed by the big-endian payload of the
 * fixed-width value type selected by the {@code valueType} token. A direct
 * buffer holds no arbitrary objects, so the type token selects the slot payload
 * width, the role the C++ {@code V} template parameter carries. Zero-filled
 * slots read as vacant.
 *
 * <p>Port of the C++ {@code tagma::CoordSpaceM<N, V>} in
 * {@code sw/cpp/tagma_core/include/tagma_core/coord_space_m.h}; the underlying
 * behavior mirrors the Rust {@code CoordSpaceM<N, V>} in
 * {@code sw/rust/core/src/coord_space_m.rs}.
 *
 * @param <V> the stored value type: one of {@code Boolean}, {@code Byte},
 *            {@code Character}, {@code Short}, {@code Integer}, {@code Float},
 *            {@code Long}, {@code Double}
 */
public final class CoordSpaceM<V> implements AutoCloseable {

    /** The only supported depth, 3 characters: 11,172^3 slots. */
    public static final int SUPPORTED_DEPTH = 3;

    /** The fixed slot count of the supported depth: 11172^3 = 1,394,417,360,448. */
    public static final long SLOT_COUNT = (long) Coord.N_VALID * Coord.N_VALID * Coord.N_VALID;

    /** The presence byte of an engaged slot. */
    private static final byte ENGAGED = 1;

    /** The presence byte of a vacant slot. */
    private static final byte VACANT = 0;

    private final int depth;
    private final ValueType valueType;
    private final int stride;
    private final int windowBytes;
    private final int windowCount;
    private final long slotsPerWindow;
    private final long slotRegionBytes;
    private final ByteBuffer[] windows;
    private long size;
    private boolean closed;

    /**
     * Creates an empty space. The slot region is reserved logically only: no
     * window is allocated until a value is placed.
     *
     * @throws IllegalArgumentException when {@code depth} is not
     *         {@link #SUPPORTED_DEPTH} or the value type is not a supported
     *         fixed-width type
     */
    public CoordSpaceM(int depth, Class<V> valueType) {
        if (depth != SUPPORTED_DEPTH) {
            throw new IllegalArgumentException(
                    "CoordSpaceM: unsupported depth " + depth + " (only " + SUPPORTED_DEPTH + " is supported)");
        }
        this.depth = depth;
        this.valueType = typeOf(Objects.requireNonNull(valueType, "valueType"));
        this.stride = 1 + this.valueType.payloadBytes;
        this.windowBytes = Integer.MAX_VALUE / stride * stride;
        this.slotsPerWindow = (long) windowBytes / stride;
        this.slotRegionBytes = SLOT_COUNT * stride;
        this.windowCount = (int) ((slotRegionBytes + windowBytes - 1) / windowBytes);
        this.windows = new ByteBuffer[windowCount];
    }

    // ------------------------------------------------------------------
    // Shape and state
    // ------------------------------------------------------------------

    /** The depth, always {@link #SUPPORTED_DEPTH}. */
    public int depth() {
        return depth;
    }

    /** The fixed slot count, always {@link #SLOT_COUNT}. */
    public long capacity() {
        return SLOT_COUNT;
    }

    /** The number of occupied slots; zero once the space is closed. */
    public long size() {
        return size;
    }

    /** Whether no slots are occupied; true once the space is closed. */
    public boolean isEmpty() {
        return size == 0;
    }

    /**
     * The number of materialized windows. Package-private, so the tests can pin
     * the lazy allocation without adding public surface the references do not
     * have.
     */
    int allocatedWindows() {
        int allocated = 0;
        for (ByteBuffer window : windows) {
            if (window != null) {
                allocated += 1;
            }
        }
        return allocated;
    }

    /** Displays the space, mirroring the shape of the Rust {@code Debug} impl. */
    @Override
    public String toString() {
        return "CoordSpaceM { N: " + depth + ", len: " + size + ", value: " + valueType
                + ", closed: " + closed + " }";
    }

    // ------------------------------------------------------------------
    // Slot access
    // ------------------------------------------------------------------

    /**
     * The value at {@code path}.
     *
     * @return empty when the slot is vacant
     * @throws IllegalArgumentException when {@code path.length() != depth()}
     * @throws IllegalStateException when the space is closed
     */
    public Optional<V> atPath(CoordPath path) {
        requireOpen();
        long index = linearIndex(path);
        ByteBuffer window = windowFor(index, false);
        if (window == null) {
            return Optional.empty();
        }
        int offset = slotOffset(index);
        if (window.get(offset) == VACANT) {
            return Optional.empty();
        }
        return Optional.of(slotValue(decode(window, offset + 1)));
    }

    /**
     * Places {@code value} at {@code path}, materializing the window that holds
     * it.
     *
     * @return the previous value when the slot was occupied
     * @throws IllegalArgumentException when {@code path.length() != depth()}
     * @throws IllegalStateException when the space is closed
     * @throws NullPointerException when {@code value} is null
     * @throws OutOfMemoryError when the window cannot be reserved against
     *         {@code -XX:MaxDirectMemorySize}
     */
    public Optional<V> placePath(CoordPath path, V value) {
        requireOpen();
        Objects.requireNonNull(value, "null value");
        long index = linearIndex(path);
        ByteBuffer window = windowFor(index, true);
        int offset = slotOffset(index);
        Optional<V> previous = window.get(offset) == VACANT
                ? Optional.empty()
                : Optional.of(slotValue(decode(window, offset + 1)));
        encode(window, offset + 1, value);
        window.put(offset, ENGAGED);
        if (previous.isEmpty()) {
            size += 1;
        }
        return previous;
    }

    /**
     * Removes the value at {@code path}.
     *
     * @return the removed value when the slot was occupied
     * @throws IllegalArgumentException when {@code path.length() != depth()}
     * @throws IllegalStateException when the space is closed
     */
    public Optional<V> vacatePath(CoordPath path) {
        requireOpen();
        long index = linearIndex(path);
        ByteBuffer window = windowFor(index, false);
        if (window == null) {
            return Optional.empty();
        }
        int offset = slotOffset(index);
        if (window.get(offset) == VACANT) {
            return Optional.empty();
        }
        V previous = slotValue(decode(window, offset + 1));
        window.put(offset, VACANT);
        size -= 1;
        return Optional.of(previous);
    }

    /**
     * Removes all values by dropping every materialized window, so the space
     * reads as freshly zero-filled memory and the buffers become unreachable.
     * The C++ reference discards the pages with {@code madvise(MADV_DONTNEED)}
     * or a {@code MAP_FIXED} remap while keeping its single mapping; the
     * observable state is the same, empty with the fixed capacity unchanged.
     *
     * @throws IllegalStateException when the space is closed
     */
    public void clear() {
        requireOpen();
        Arrays.fill(windows, null);
        size = 0;
    }

    /**
     * Retains only the values satisfying {@code keep}; failing entries are
     * vacated. The C++ and Rust {@code CoordSpaceM} have no retain; this
     * mirrors the family operation defined by {@code CoordSpace::retain} in
     * {@code sw/cpp/tagma_core/include/tagma_core/coord_space.h} and by
     * {@code CoordSpaceN<1>::retain} in {@code sw/rust/core/src/coord_space_n.rs},
     * applied to the materialized windows.
     *
     * <p>The walk visits every slot of every materialized window in window
     * order and never allocates, so its cost is proportional to the
     * materialized windows rather than to the number of entries.
     *
     * @throws IllegalStateException when the space is closed
     */
    public void retain(BiPredicate<CoordPath, V> keep) {
        requireOpen();
        Objects.requireNonNull(keep, "keep");
        for (int number = 0; number < windowCount; number++) {
            ByteBuffer window = windows[number];
            if (window == null) {
                continue;
            }
            long firstSlot = firstSlotOfWindow(number);
            int slots = window.capacity() / stride;
            for (int slot = 0; slot < slots; slot++) {
                int offset = slot * stride;
                if (window.get(offset) == VACANT) {
                    continue;
                }
                V value = slotValue(decode(window, offset + 1));
                if (!keep.test(pathFromIndex(firstSlot + slot), value)) {
                    window.put(offset, VACANT);
                    size -= 1;
                }
            }
        }
    }

    // ------------------------------------------------------------------
    // Lifecycle
    // ------------------------------------------------------------------

    /**
     * Drops the materialized windows and closes the space: {@link #size()}
     * becomes zero and every slot operation throws
     * {@link IllegalStateException}. Java cannot release off-heap memory
     * deterministically, so the dropped buffers stay accounted against
     * {@code -XX:MaxDirectMemorySize} until the {@code Cleaner} of the buffer
     * implementation runs. Closing an already closed space does nothing.
     */
    @Override
    public void close() {
        if (closed) {
            return;
        }
        closed = true;
        Arrays.fill(windows, null);
        size = 0;
    }

    // ------------------------------------------------------------------
    // Internals
    // ------------------------------------------------------------------

    /**
     * The fixed-width value types a slot can hold, with the payload width in
     * bytes. A direct buffer holds no arbitrary objects, so the type token
     * selects the slot payload width instead of the C++ {@code V} template
     * parameter.
     */
    private enum ValueType {

        BOOLEAN(1),
        BYTE(1),
        CHARACTER(2),
        SHORT(2),
        INTEGER(4),
        FLOAT(4),
        LONG(8),
        DOUBLE(8);

        private final int payloadBytes;

        ValueType(int payloadBytes) {
            this.payloadBytes = payloadBytes;
        }
    }

    private static ValueType typeOf(Class<?> type) {
        if (type == Boolean.class) {
            return ValueType.BOOLEAN;
        }
        if (type == Byte.class) {
            return ValueType.BYTE;
        }
        if (type == Character.class) {
            return ValueType.CHARACTER;
        }
        if (type == Short.class) {
            return ValueType.SHORT;
        }
        if (type == Integer.class) {
            return ValueType.INTEGER;
        }
        if (type == Float.class) {
            return ValueType.FLOAT;
        }
        if (type == Long.class) {
            return ValueType.LONG;
        }
        if (type == Double.class) {
            return ValueType.DOUBLE;
        }
        throw new IllegalArgumentException("CoordSpaceM unsupported value type: " + type.getName());
    }

    private void requireOpen() {
        if (closed) {
            throw new IllegalStateException("CoordSpaceM: space is closed");
        }
    }

    private long linearIndex(CoordPath path) {
        Objects.requireNonNull(path, "path");
        if (path.length() != depth) {
            throw new IllegalArgumentException(
                    "CoordSpaceM path length " + path.length() + " does not match depth " + depth);
        }
        long index = 0;
        for (Coord coord : path.coords()) {
            index = index * Coord.N_VALID + coord.index();
        }
        return index;
    }

    private static CoordPath pathFromIndex(long index) {
        Coord[] coords = new Coord[SUPPORTED_DEPTH];
        long remainder = index;
        for (int i = SUPPORTED_DEPTH - 1; i >= 0; i--) {
            coords[i] = Coord.raw((int) (remainder % Coord.N_VALID));
            remainder /= Coord.N_VALID;
        }
        return CoordPath.fromArray(coords);
    }

    private int windowNumber(long index) {
        return (int) (index / slotsPerWindow);
    }

    private long firstSlotOfWindow(int number) {
        return (long) number * slotsPerWindow;
    }

    /** The byte extent of a window, always a whole number of slots. */
    private int windowExtent(int number) {
        return (int) Math.min(windowBytes, slotRegionBytes - (long) number * windowBytes);
    }

    /**
     * Returns the materialized window holding {@code index}, or null when the
     * window holds no value yet. With {@code create} set the window is
     * allocated as zero-filled off-heap memory on first touch.
     */
    private ByteBuffer windowFor(long index, boolean create) {
        int number = windowNumber(index);
        ByteBuffer window = windows[number];
        if (window == null && create) {
            window = ByteBuffer.allocateDirect(windowExtent(number));
            windows[number] = window;
        }
        return window;
    }

    private int slotOffset(long index) {
        return (int) ((index - firstSlotOfWindow(windowNumber(index))) * stride);
    }

    private void encode(ByteBuffer out, int offset, V value) {
        switch (valueType) {
            case BOOLEAN -> out.put(offset, (Boolean) value ? (byte) 1 : (byte) 0);
            case BYTE -> out.put(offset, (Byte) value);
            case CHARACTER -> out.putChar(offset, (Character) value);
            case SHORT -> out.putShort(offset, (Short) value);
            case INTEGER -> out.putInt(offset, (Integer) value);
            case FLOAT -> out.putFloat(offset, (Float) value);
            case LONG -> out.putLong(offset, (Long) value);
            case DOUBLE -> out.putDouble(offset, (Double) value);
        }
    }

    private Object decode(ByteBuffer in, int offset) {
        return switch (valueType) {
            case BOOLEAN -> Boolean.valueOf(in.get(offset) != VACANT);
            case BYTE -> in.get(offset);
            case CHARACTER -> in.getChar(offset);
            case SHORT -> in.getShort(offset);
            case INTEGER -> in.getInt(offset);
            case FLOAT -> in.getFloat(offset);
            case LONG -> in.getLong(offset);
            case DOUBLE -> in.getDouble(offset);
        };
    }

    /** Reads the value of a slot. One suppression covers the class; Java generics are erased. */
    @SuppressWarnings("unchecked")
    private V slotValue(Object decoded) {
        return (V) decoded;
    }
}
