package org.ssccs.syntagma.bench;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.io.ByteArrayOutputStream;
import java.io.PrintStream;
import java.nio.charset.StandardCharsets;
import java.nio.file.Path;
import java.util.OptionalInt;

import org.junit.jupiter.api.Test;

/**
 * Cases for the command line of {@link BenchCli}: the flag contract of the C++
 * {@code main} in {@code sw/cpp/bench/bench.cpp}, the Java smoke flags added on
 * top of it, and the argument-loop behavior the port keeps or deliberately
 * rejects.
 */
class BenchCliTest {

    @Test
    void defaultsMatchTheCppContract() {
        BenchCli.Options options = BenchCli.parse(new String[] {});

        assertTrue(options.jsonPath().isEmpty(), "json path");
        assertEquals(BenchCli.UNKNOWN, options.commit(), "commit");
        assertEquals(BenchCli.UNKNOWN, options.timestamp(), "timestamp");
        assertTrue(options.iterations().isEmpty(), "iterations override");
        assertTrue(options.rounds().isEmpty(), "rounds override");
        assertFalse(options.quick(), "quick");
        assertFalse(options.help(), "help");
    }

    @Test
    void parsesTheCppFlags() {
        BenchCli.Options options = BenchCli.parse(new String[] {
            "--json", "/tmp/bench-result.json", "--commit", "abc1234", "--timestamp", "20260101-000000"});

        assertEquals(Path.of("/tmp/bench-result.json"), options.jsonPath().orElseThrow(), "json path");
        assertEquals("abc1234", options.commit(), "commit");
        assertEquals("20260101-000000", options.timestamp(), "timestamp");
    }

    @Test
    void parsesTheSmokeFlags() {
        BenchCli.Options options = BenchCli.parse(new String[] {"--iterations", "5", "--rounds", "2"});

        assertEquals(OptionalInt.of(5), options.iterations(), "iterations");
        assertEquals(OptionalInt.of(2), options.rounds(), "rounds");
        assertFalse(options.quick(), "quick");
    }

    @Test
    void detectsQuick() {
        assertTrue(BenchCli.parse(new String[] {"--quick"}).quick(), "quick");
    }

    @Test
    void detectsHelp() {
        assertTrue(BenchCli.parse(new String[] {"--help"}).help(), "help");
    }

    @Test
    void ignoresUnknownArgumentsLikeTheCppLoop() {
        BenchCli.Options options = BenchCli.parse(new String[] {"--unknown", "value", "--commit", "abc1234"});

        assertEquals("abc1234", options.commit(), "commit");
        assertTrue(options.jsonPath().isEmpty(), "json path");
    }

    @Test
    void rejectsFlagsWithoutValues() {
        for (String flag : new String[] {"--json", "--commit", "--timestamp", "--iterations", "--rounds"}) {
            IllegalArgumentException error = assertThrows(IllegalArgumentException.class,
                    () -> BenchCli.parse(new String[] {flag}), flag);
            assertTrue(error.getMessage().contains("requires a value"), error.getMessage());
        }
    }

    @Test
    void rejectsNonNumericCounts() {
        IllegalArgumentException error = assertThrows(IllegalArgumentException.class,
                () -> BenchCli.parse(new String[] {"--iterations", "many"}));
        assertTrue(error.getMessage().contains("requires an integer"), error.getMessage());
    }

    @Test
    void rejectsNonPositiveCounts() {
        for (String count : new String[] {"0", "-3"}) {
            assertThrows(IllegalArgumentException.class,
                    () -> BenchCli.parse(new String[] {"--rounds", count}), count);
        }
    }

    @Test
    void rejectsAnUnusableJsonPath() {
        assertThrows(IllegalArgumentException.class,
                () -> BenchCli.parse(new String[] {"--json", "bad\u0000path"}));
    }

    @Test
    void usageNamesTheProgramAndEveryFlag() {
        ByteArrayOutputStream buffer = new ByteArrayOutputStream();
        BenchCli.printUsage(new PrintStream(buffer, true, StandardCharsets.UTF_8));
        String usage = buffer.toString(StandardCharsets.UTF_8);

        assertTrue(usage.contains("Usage: " + BenchCli.PROGRAM), usage);
        for (String flag : new String[] {
            "--json", "--commit", "--timestamp", "--iterations", "--rounds", "--quick", "--help"}) {
            assertTrue(usage.contains(flag), usage);
        }
    }
}
