package org.ssccs.syntagma.sec;

import java.nio.charset.StandardCharsets;
import java.util.function.Consumer;
import org.ssccs.syntagma.core.Coord;
import org.ssccs.syntagma.core.CoordPath;

/**
 * Shared fixtures of the translated C++ suites
 * {@code sw/cpp/tagma_sec/tests}: byte and path builders and the
 * run-against-both-stacks driver that make the stack swap the contract under
 * test.
 */
final class Fixtures {

    private Fixtures() {
    }

    /** Bytes of an ASCII string, mirroring the C++ {@code bytes} helper. */
    static byte[] bytes(String text) {
        return text.getBytes(StandardCharsets.UTF_8);
    }

    /** A path from raw coordinate indices, mirroring the C++ {@code path} helper. */
    static CoordPath path(int... indexes) {
        Coord[] coords = new Coord[indexes.length];
        for (int i = 0; i < indexes.length; i++) {
            coords[i] = Coord.fromIndex(indexes[i]).orElseThrow();
        }
        return CoordPath.fromArray(coords);
    }

    /** Runs the given scenario against both stack implementations. */
    static void withBothStacks(Consumer<SecStack> scenario) {
        scenario.accept(SecStack.legacy());
        scenario.accept(SecStack.delos());
    }
}
