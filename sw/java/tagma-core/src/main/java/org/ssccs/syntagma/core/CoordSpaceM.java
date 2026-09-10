package org.ssccs.syntagma.core;

import java.io.IOException;
import java.io.UncheckedIOException;
import java.nio.ByteBuffer;
import java.nio.MappedByteBuffer;
import java.nio.channels.FileChannel;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardOpenOption;
import java.util.HashMap;
import java.util.Map;
import java.util.Objects;
import java.util.Optional;
import java.util.function.BiPredicate;

/**
 * A dense, file-mapped direct-address space of depth 3: the Java counterpart
 * of the mmap-backed C++ {@code tagma::CoordSpaceM<N, V>}.
 *
 * <p>Addressing is direct: a {@link CoordPath} maps to a linear index by the
 * base-11172 fold, so placement and lookup are a single slot access, with no
 * hashing and no collisions. The slot count is fixed at 11,172^3
 * (1,394,417,360,448) and the constructor rejects every other depth.
 *
 * <p>Platform difference. The C++ and Rust references reserve the slot region
 * with an anonymous {@code mmap} ({@code MAP_NORESERVE} on Linux) and are
 * therefore Unix-only. Java has no portable anonymous mapping;
 * {@link FileChannel#map} and {@link MappedByteBuffer} replace it, which keeps
 * the class portable and makes the region file-backed. That is the one
 * capability the native implementations do not have: the slots persist in the
 * file, so a space reopened over the same path observes the previous contents,
 * and {@link #close()} flushes them.
 *
 * <p>A {@code MappedByteBuffer} is limited to {@code Integer.MAX_VALUE} bytes,
 * so the slot region is mapped in windows sized to the largest multiple of the
 * slot stride that fits. Each window is mapped on first use and retained until
 * {@link #close()}; the whole region needs about 6,500 windows for the widest
 * value type, which stays inside the mapping budget of a 64-bit process.
 *
 * <p>Page reclamation. {@link #clear()} trims the slot region and re-extends it
 * lazily, so storage is reclaimed rather than retained. The C++ reference
 * discards the pages with {@code madvise(MADV_DONTNEED)} or a {@code MAP_FIXED}
 * remap while keeping the mapping. The observable state after {@code clear} is
 * identical: empty, same fixed capacity, same file.
 *
 * <p>The C++ port omits the Rust {@code Clone} and {@code PartialEq}, because a
 * byte-wise copy or comparison would walk the multi-terabyte region page by
 * page. The Java port keeps the same surface: no copy method, identity
 * equality.
 *
 * <p>Slot encoding: one presence byte followed by the big-endian payload of the
 * fixed-width value type selected by the {@code valueType} token. Zero-filled
 * file regions read as vacant slots, mirroring the C++ invariant that a slot
 * whose bytes are all zero is disengaged. One page of header precedes the slot
 * region and holds the magic, the format version, the depth, the slot stride,
 * the value-type tag, the engaged count and a bitmap of the windows that have
 * held a value, which bounds {@link #retain} to the written windows. The header
 * is refreshed on every mutation and validated on open.
 *
 * <p>The file is owned by a single writer: header validation rejects a foreign
 * file and a file written for another depth, stride or value type, and
 * concurrent access to one file from several instances is not synchronized.
 * There is no write-ahead protocol either, so an abrupt termination can leave
 * the persisted engaged count ahead of or behind the slots; {@link #force()}
 * and {@link #close()} flush the mapping but do not validate it.
 *
 * <p>File failures surface as {@link UncheckedIOException}, the Java
 * counterpart of the {@code std::runtime_error} the C++ constructor raises when
 * {@code mmap} fails. The slot accessors therefore keep the reference
 * signatures free of checked exceptions.
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

    /** The header size in bytes, one page. The slot region starts here. */
    private static final int HEADER_BYTES = 4096;

    /** Header magic, "TGM1". */
    private static final byte[] MAGIC = {(byte) 'T', (byte) 'G', (byte) 'M', (byte) '1'};

    private static final int VERSION_OFFSET = 4;

    private static final int DEPTH_OFFSET = 8;

    private static final int STRIDE_OFFSET = 12;

    private static final int TYPE_OFFSET = 16;

    private static final int ENGAGED_OFFSET = 24;

    /** Start of the written-window bitmap inside the header page. */
    private static final int BITMAP_OFFSET = 64;

    private static final int FORMAT_VERSION = 1;

    /** The presence byte of an engaged slot. */
    private static final byte ENGAGED = 1;

    /** The presence byte of a vacant slot. */
    private static final byte VACANT = 0;

    private final int depth;
    private final ValueType valueType;
    private final int stride;
    private final int windowBytes;
    private final int windowCount;
    private final int bitmapBytes;
    private final long slotsPerWindow;
    private final long slotRegionBytes;
    private final Path file;
    private final boolean temporary;
    private final FileChannel channel;
    private final MappedByteBuffer header;
    private final Map<Integer, MappedByteBuffer> windows = new HashMap<>();
    private long size;
    private boolean closed;

    /**
     * Creates an empty space over a new temporary file, mirroring the
     * anonymous-mapping C++ default constructor. The file is deleted when the
     * space is closed.
     *
     * @throws IllegalArgumentException when {@code depth} is not
     *         {@link #SUPPORTED_DEPTH} or the value type is not a supported
     *         fixed-width type
     */
    public CoordSpaceM(int depth, Class<V> valueType) {
        this(depth, valueType, null, true);
    }

    /**
     * Creates or reopens a space over {@code file}. An existing empty file is
     * initialized; an existing space file is validated and its contents are
     * preserved, which is how persistence across reopen is observed.
     *
     * @throws IllegalArgumentException when {@code depth} is not
     *         {@link #SUPPORTED_DEPTH}, the value type is not supported, or the
     *         file does not hold a matching space
     */
    public CoordSpaceM(int depth, Class<V> valueType, Path file) {
        this(depth, valueType, file, false);
    }

    private CoordSpaceM(int depth, Class<V> valueType, Path file, boolean temporary) {
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
        this.bitmapBytes = (windowCount + 7) / 8;
        if (BITMAP_OFFSET + bitmapBytes > HEADER_BYTES) {
            throw new IllegalStateException(
                    "CoordSpaceM: window bitmap does not fit the header page for stride " + stride);
        }
        this.temporary = temporary;
        Path target = temporary ? createTemporaryFile() : Objects.requireNonNull(file, "file");
        this.file = target;
        try {
            this.channel = FileChannel.open(
                    target, StandardOpenOption.READ, StandardOpenOption.WRITE, StandardOpenOption.CREATE);
            long length = channel.size();
            if (length == 0) {
                channel.write(newHeader(), 0);
            } else if (length < HEADER_BYTES) {
                throw new IllegalArgumentException(
                        "CoordSpaceM: file is too short to hold the header: " + target);
            }
            this.header = channel.map(FileChannel.MapMode.READ_WRITE, 0, HEADER_BYTES);
            this.size = length == 0 ? 0 : readHeader();
        } catch (IOException e) {
            throw new UncheckedIOException("CoordSpaceM: cannot open " + target, e);
        }
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

    /** The number of occupied slots. */
    public long size() {
        return size;
    }

    /** Whether no slots are occupied. */
    public boolean isEmpty() {
        return size == 0;
    }

    /** The backing file, which carries the slot contents across reopen. */
    public Path file() {
        return file;
    }

    /** Displays the space, mirroring the shape of the Rust {@code Debug} impl. */
    @Override
    public String toString() {
        return "CoordSpaceM { N: " + depth + ", len: " + size + ", value: " + valueType
                + ", file: " + file + " }";
    }

    // ------------------------------------------------------------------
    // Slot access
    // ------------------------------------------------------------------

    /**
     * The value at {@code path}.
     *
     * @return empty when the slot is vacant
     * @throws IllegalArgumentException when {@code path.length() != depth()}
     */
    public Optional<V> atPath(CoordPath path) {
        requireOpen();
        long index = linearIndex(path);
        MappedByteBuffer window = windowFor(index, false);
        if (window == null) {
            return Optional.empty();
        }
        int offset = slotOffset(index);
        if (offset + stride > window.capacity() || window.get(offset) == VACANT) {
            return Optional.empty();
        }
        return Optional.of(slotValue(decode(window, offset + 1)));
    }

    /**
     * Places {@code value} at {@code path}.
     *
     * @return the previous value when the slot was occupied
     * @throws IllegalArgumentException when {@code path.length() != depth()}
     * @throws NullPointerException when {@code value} is null
     */
    public Optional<V> placePath(CoordPath path, V value) {
        requireOpen();
        Objects.requireNonNull(value, "null value");
        long index = linearIndex(path);
        MappedByteBuffer window = windowFor(index, true);
        int offset = slotOffset(index);
        Optional<V> previous = window.get(offset) == VACANT
                ? Optional.empty()
                : Optional.of(slotValue(decode(window, offset + 1)));
        encode(window, offset + 1, value);
        window.put(offset, ENGAGED);
        markWindow(windowNumber(index));
        if (previous.isEmpty()) {
            setSize(size + 1);
        }
        return previous;
    }

    /**
     * Removes the value at {@code path}.
     *
     * @return the removed value when the slot was occupied
     * @throws IllegalArgumentException when {@code path.length() != depth()}
     */
    public Optional<V> vacatePath(CoordPath path) {
        requireOpen();
        long index = linearIndex(path);
        MappedByteBuffer window = windowFor(index, false);
        if (window == null) {
            return Optional.empty();
        }
        int offset = slotOffset(index);
        if (offset + stride > window.capacity() || window.get(offset) == VACANT) {
            return Optional.empty();
        }
        V previous = slotValue(decode(window, offset + 1));
        window.put(offset, VACANT);
        setSize(size - 1);
        return Optional.of(previous);
    }

    /**
     * Removes all values. The backing file and the fixed capacity are kept;
     * the slot region is trimmed and re-extended lazily by the next placement.
     */
    public void clear() {
        requireOpen();
        windows.clear();
        try {
            channel.truncate(HEADER_BYTES);
        } catch (IOException e) {
            throw new UncheckedIOException("CoordSpaceM: cannot clear " + file, e);
        }
        for (int i = 0; i < bitmapBytes; i++) {
            header.put(BITMAP_OFFSET + i, VACANT);
        }
        setSize(0);
    }

    /**
     * Retains only the values satisfying {@code keep}; failing entries are
     * vacated. The C++ and Rust {@code CoordSpaceM} have no retain; this
     * mirrors the family operation defined by {@code CoordSpace::retain} in
     * {@code sw/cpp/tagma_core/include/tagma_core/coord_space.h} and by
     * {@code CoordSpaceN<1>::retain} in {@code sw/rust/core/src/coord_space_n.rs},
     * applied to the mapped slot region.
     *
     * <p>The walk covers every slot of every window the file already holds a
     * written slot in, so its cost is proportional to those windows rather than
     * to the number of entries.
     */
    public void retain(BiPredicate<CoordPath, V> keep) {
        requireOpen();
        Objects.requireNonNull(keep, "keep");
        for (int number = 0; number < windowCount; number++) {
            if (!isWindowMarked(number)) {
                continue;
            }
            MappedByteBuffer window = windowFor(firstSlotOfWindow(number), false);
            if (window == null) {
                throw new IllegalStateException(
                        "CoordSpaceM: window " + number + " is marked but not backed by " + file);
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
                    setSize(size - 1);
                }
            }
        }
    }

    // ------------------------------------------------------------------
    // Lifecycle
    // ------------------------------------------------------------------

    /** Flushes the mapped header and slot windows to the backing file. */
    public void force() {
        requireOpen();
        try {
            forceBuffers();
        } catch (IOException e) {
            throw new UncheckedIOException("CoordSpaceM: cannot flush " + file, e);
        }
    }

    /**
     * Flushes the mapped windows, closes the channel and deletes the file when
     * the space was created over a temporary file. Later slot access throws
     * {@link IllegalStateException}.
     */
    @Override
    public void close() {
        if (closed) {
            return;
        }
        closed = true;
        try {
            forceBuffers();
            windows.clear();
            channel.close();
            if (temporary) {
                Files.deleteIfExists(file);
            }
        } catch (IOException e) {
            throw new UncheckedIOException("CoordSpaceM: cannot close " + file, e);
        }
    }

    // ------------------------------------------------------------------
    // Internals
    // ------------------------------------------------------------------

    /**
     * The fixed-width value types a slot can hold, with the on-disk tag and the
     * payload width in bytes. Java cannot store arbitrary objects in mapped
     * bytes, so the type token selects the slot payload width instead of the
     * C++ {@code V} template parameter.
     */
    private enum ValueType {

        BOOLEAN(1, 1),
        BYTE(2, 1),
        CHARACTER(3, 2),
        SHORT(4, 2),
        INTEGER(5, 4),
        FLOAT(6, 4),
        LONG(7, 8),
        DOUBLE(8, 8);

        private final int tag;
        private final int payloadBytes;

        ValueType(int tag, int payloadBytes) {
            this.tag = tag;
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

    private static Path createTemporaryFile() {
        try {
            Path file = Files.createTempFile("coord-space-m-", ".bin");
            file.toFile().deleteOnExit();
            return file;
        } catch (IOException e) {
            throw new UncheckedIOException("CoordSpaceM: cannot create a temporary file", e);
        }
    }

    private ByteBuffer newHeader() {
        ByteBuffer buffer = ByteBuffer.allocate(HEADER_BYTES);
        buffer.put(MAGIC);
        buffer.putInt(FORMAT_VERSION);
        buffer.putInt(SUPPORTED_DEPTH);
        buffer.putInt(stride);
        buffer.putInt(valueType.tag);
        buffer.putInt(0);
        buffer.putLong(0L);
        buffer.flip();
        return buffer;
    }

    private long readHeader() {
        for (int i = 0; i < MAGIC.length; i++) {
            if (header.get(i) != MAGIC[i]) {
                throw new IllegalArgumentException("CoordSpaceM: not a CoordSpaceM file: " + file);
            }
        }
        int version = header.getInt(VERSION_OFFSET);
        if (version != FORMAT_VERSION) {
            throw new IllegalArgumentException(
                    "CoordSpaceM: unsupported format version " + version + " in " + file);
        }
        int storedDepth = header.getInt(DEPTH_OFFSET);
        int storedStride = header.getInt(STRIDE_OFFSET);
        int storedTag = header.getInt(TYPE_OFFSET);
        if (storedDepth != depth || storedStride != stride || storedTag != valueType.tag) {
            throw new IllegalArgumentException(
                    "CoordSpaceM: file layout mismatch (depth " + storedDepth + ", stride " + storedStride
                            + ", value tag " + storedTag + ") in " + file);
        }
        return header.getLong(ENGAGED_OFFSET);
    }

    private void requireOpen() {
        if (closed) {
            throw new IllegalStateException("CoordSpaceM: space is closed: " + file);
        }
    }

    private void setSize(long newSize) {
        size = newSize;
        header.putLong(ENGAGED_OFFSET, newSize);
    }

    /** Records that {@code number} has held a value at least once since the last clear. */
    private void markWindow(int number) {
        int index = BITMAP_OFFSET + (number >> 3);
        header.put(index, (byte) (header.get(index) | (1 << (number & 7))));
    }

    private boolean isWindowMarked(int number) {
        return (header.get(BITMAP_OFFSET + (number >> 3)) & (1 << (number & 7))) != 0;
    }

    private void forceBuffers() throws IOException {
        header.force();
        for (MappedByteBuffer window : windows.values()) {
            window.force();
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

    /**
     * Returns the mapped window holding {@code index}, mapping it on demand.
     * When {@code create} is set the file is extended to the end of the window
     * first; otherwise a window that the file does not cover yet is reported as
     * absent, which reads as a region of vacant slots.
     */
    private MappedByteBuffer windowFor(long index, boolean create) {
        int number = windowNumber(index);
        MappedByteBuffer window = windows.get(number);
        if (window != null) {
            return window;
        }
        long offset = HEADER_BYTES + (long) number * windowBytes;
        long extent = Math.min(windowBytes, slotRegionBytes - (long) number * windowBytes);
        try {
            long length = channel.size();
            if (offset >= length) {
                if (!create) {
                    return null;
                }
                // Extend the sparse file to the end of the window.
                channel.write(ByteBuffer.wrap(new byte[] {VACANT}), offset + extent - 1);
                length = channel.size();
            }
            long mapped = Math.min(extent, length - offset);
            window = channel.map(FileChannel.MapMode.READ_WRITE, offset, mapped);
        } catch (IOException e) {
            throw new UncheckedIOException("CoordSpaceM: cannot map window " + number + " of " + file, e);
        }
        windows.put(number, window);
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
