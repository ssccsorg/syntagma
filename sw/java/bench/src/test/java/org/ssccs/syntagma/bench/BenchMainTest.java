package org.ssccs.syntagma.bench;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.io.ByteArrayOutputStream;
import java.io.PrintStream;
import java.nio.charset.StandardCharsets;

import org.junit.jupiter.api.Test;

/**
 * Cases for the entry point {@link BenchMain} that do not run a scenario: the
 * help path and the usage-error path, which stop before the suite starts and
 * therefore keep the test phase fast. The measurement itself runs through the
 * exec plugin, not through {@code mvn verify}.
 */
class BenchMainTest {

    private final ByteArrayOutputStream out = new ByteArrayOutputStream();

    private final ByteArrayOutputStream err = new ByteArrayOutputStream();

    private int run(String[] args) {
        return BenchMain.run(args, new PrintStream(out, true, StandardCharsets.UTF_8),
                new PrintStream(err, true, StandardCharsets.UTF_8));
    }

    private String stdout() {
        return out.toString(StandardCharsets.UTF_8);
    }

    private String stderr() {
        return err.toString(StandardCharsets.UTF_8);
    }

    @Test
    void helpPrintsTheUsageAndSucceeds() {
        assertEquals(0, run(new String[] {"--help"}), "status");
        assertTrue(stdout().contains("Usage: " + BenchCli.PROGRAM), stdout());
    }

    @Test
    void aUsageErrorFailsWithStatusTwo() {
        assertEquals(2, run(new String[] {"--json"}), "status");
        assertTrue(stderr().contains("--json requires a value"), stderr());
        assertTrue(stderr().contains("Usage: " + BenchCli.PROGRAM), stderr());
        assertEquals("", stdout(), "the suite must not start");
    }
}
