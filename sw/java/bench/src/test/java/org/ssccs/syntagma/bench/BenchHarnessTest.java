package org.ssccs.syntagma.bench;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.io.ByteArrayOutputStream;
import java.io.PrintStream;
import java.nio.charset.StandardCharsets;
import java.util.OptionalInt;

import org.junit.jupiter.api.Test;

/**
 * Cases for {@link BenchHarness}: the call shape of a measured scenario, the
 * result it records, the blackhole sink the work reaches, and the console line
 * it prints. The timings themselves are not asserted, only the structure and
 * the counters, so the cases stay deterministic.
 */
class BenchHarnessTest {

    /** A harness and the console text it printed so far. */
    private record Measured(BenchHarness harness, ByteArrayOutputStream buffer) {

        String console() {
            return buffer.toString(StandardCharsets.UTF_8);
        }
    }

    private static Measured measured(BenchProfile profile) {
        ByteArrayOutputStream buffer = new ByteArrayOutputStream();
        PrintStream out = new PrintStream(buffer, true, StandardCharsets.UTF_8);
        return new Measured(new BenchHarness(profile, out), buffer);
    }

    /** The profile every counting case uses: three iterations over two rounds, two warmups. */
    private static BenchProfile counting() {
        return new BenchProfile(OptionalInt.of(3), OptionalInt.of(2), 2);
    }

    @Test
    void runsTheWarmupThenTheRoundsWithTheirOwnWarmup() {
        BenchHarness harness = measured(counting()).harness();
        int[] calls = {0};

        harness.run("scenario", 7, 5, () -> calls[0] += 1);

        // Two explicit warmup calls, then two rounds of one per-round warmup
        // call and three timed iterations; the overrides win over the bases.
        assertEquals(2 + 2 * (1 + 3), calls[0], "calls");
    }

    @Test
    void recordsOneResultPerScenarioInExecutionOrder() {
        BenchHarness harness = measured(BenchProfile.quick()).harness();

        harness.run("first", 1, 1, () -> harness.sink().add(1));
        harness.run("second", 1, 1, () -> harness.sink().add(2));

        assertEquals(2, harness.results().size(), "result count");
        assertEquals("first", harness.results().get(0).name(), "first name");
        assertEquals("second", harness.results().get(1).name(), "second name");
    }

    @Test
    void accumulatesTheSinkAcrossWarmupAndTimedCalls() {
        BenchHarness harness = measured(counting()).harness();

        harness.run("sink", 1, 1, () -> harness.sink().add(7));

        // 2 warmup calls + 2 rounds of (1 per-round warmup + 3 timed calls).
        assertEquals(10 * 7, harness.sink().sum(), "sink sum");
    }

    @Test
    void reportsFiniteStatisticsAndPrintsOneLinePerScenario() {
        Measured measured = measured(BenchProfile.standard());

        measured.harness().run("printed", 1, 1, () -> measured.harness().sink().add(1));

        BenchResult result = measured.harness().results().get(0);
        assertTrue(Double.isFinite(result.meanNs()) && result.meanNs() >= 0.0, "mean");
        assertTrue(Double.isFinite(result.stddevNs()) && result.stddevNs() >= 0.0, "stddev");
        assertTrue(measured.console().contains("printed"), measured.console());
        assertTrue(measured.console().contains("ns/op"), measured.console());
    }

    @Test
    void resultsAreAnUnmodifiableView() {
        BenchHarness harness = measured(BenchProfile.quick()).harness();
        harness.run("scenario", 1, 1, () -> harness.sink().add(1));

        assertThrows(UnsupportedOperationException.class,
                () -> harness.results().add(new BenchResult("other", 1.0, 0.0)));
    }

    @Test
    void rejectsNonPositiveCountsAndNullArguments() {
        BenchHarness harness = measured(BenchProfile.standard()).harness();

        assertThrows(IllegalArgumentException.class, () -> harness.run("zero iterations", 0, 1, () -> { }));
        assertThrows(IllegalArgumentException.class, () -> harness.run("zero rounds", 1, 0, () -> { }));
        assertThrows(NullPointerException.class, () -> harness.run(null, 1, 1, () -> { }));
        assertThrows(NullPointerException.class, () -> harness.run("no op", 1, 1, null));
        assertThrows(NullPointerException.class, () -> new BenchHarness(null, System.out));
        assertThrows(NullPointerException.class, () -> new BenchHarness(BenchProfile.standard(), null));
    }
}
