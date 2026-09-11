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
 * Cases for {@link BenchHarness}: the call shape of a measured scenario across
 * the warmup, settling and measurement phases, the result it records, the
 * blackhole sink the work reaches, and the console line it prints.
 *
 * <p>The timing itself is not asserted, only the phase arithmetic, which the
 * policies below pin exactly: a policy whose minimum and maximum warmup rounds
 * are equal runs that many warmup rounds whatever the clock says, and a
 * tolerance of one accepts any two measurements, so the three phase tests below
 * are deterministic on a loaded machine as well.
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

    /** A policy that always runs exactly {@code rounds} warmup rounds and never cuts the budget. */
    private static BenchWarmup fixedWarmup(int rounds, int settleRounds) {
        return new BenchWarmup(rounds, rounds, 0.0, Long.MAX_VALUE, settleRounds);
    }

    @Test
    void countsTheWarmupSettlingAndMeasuredRounds() {
        BenchProfile profile = new BenchProfile(OptionalInt.of(3), OptionalInt.of(2), fixedWarmup(2, 1));
        BenchHarness harness = measured(profile).harness();
        int[] calls = {0};

        harness.run("scenario", 7, 5, () -> calls[0] += 1);

        // Two warmup rounds, one settling round and two counted rounds, three
        // iterations each; the overrides win over the base counts, and the
        // C++ per-round warmup call is no longer repeated.
        assertEquals(3 * (2 + 1 + 2), calls[0], "calls");
    }

    @Test
    void discardsTheSettlingRoundsBeforeCounting() {
        BenchProfile profile = new BenchProfile(OptionalInt.of(1), OptionalInt.of(2), fixedWarmup(1, 3));
        BenchHarness harness = measured(profile).harness();
        int[] calls = {0};

        harness.run("scenario", 1, 1, () -> calls[0] += 1);

        // One warmup round, three settling rounds and two counted rounds: the
        // settling rounds are executed and dropped.
        assertEquals(1 + 3 + 2, calls[0], "calls");
        assertEquals(1, harness.results().size(), "one result");
    }

    @Test
    void quickRunsOneWarmupRoundAndOneCountedRound() {
        BenchHarness harness = measured(BenchProfile.quick()).harness();
        int[] calls = {0};

        harness.run("scenario", 1, 1, () -> calls[0] += 1);

        assertEquals(2, calls[0], "calls");
    }

    @Test
    void warmupStopsWhenTwoConsecutiveRoundsAgree() {
        // A tolerance of one accepts any two measurements, so the phase has to
        // end at its minimum round count instead of running to the cap.
        BenchWarmup warmup = new BenchWarmup(2, 64, 1.0, Long.MAX_VALUE, 0);
        BenchProfile profile = new BenchProfile(OptionalInt.of(1), OptionalInt.of(1), warmup);
        BenchHarness harness = measured(profile).harness();
        int[] calls = {0};

        harness.run("scenario", 1, 1, () -> calls[0] += 1);

        assertEquals(2 + 1, calls[0], "calls");
    }

    @Test
    void warmupStopsWhenTheBudgetIsSpent() {
        // A zero floor and a zero budget are both spent by the first round,
        // so the phase stops there even though the cap and the tolerance ask
        // for more.
        BenchWarmup warmup = new BenchWarmup(0, 1000, 0.0, 0L, 0);
        BenchProfile profile = new BenchProfile(OptionalInt.of(1), OptionalInt.of(1), warmup);
        BenchHarness harness = measured(profile).harness();
        int[] calls = {0};

        harness.run("scenario", 1, 1, () -> calls[0] += 1);

        assertEquals(1 + 1, calls[0], "calls");
    }

    @Test
    void aScenarioWhoseWarmupNeverSettlesTerminatesAndReportsAValue() {
        // A tolerance of zero demands identical consecutive measurements, which
        // a real clock does not deliver, so the warmup phase can only end at
        // its round cap: the case the cap exists for.
        BenchWarmup neverSettles = new BenchWarmup(1, 4, 0.0, Long.MAX_VALUE, 1);
        BenchProfile profile = new BenchProfile(OptionalInt.of(1), OptionalInt.of(2), neverSettles);
        BenchHarness harness = measured(profile).harness();
        int[] calls = {0};

        harness.run("unsettled", 1, 1, () -> calls[0] += 1);

        assertTrue(calls[0] >= 1 + 1 + 2, "at least the minimum phases: " + calls[0]);
        assertTrue(calls[0] <= 4 + 1 + 2, "bounded by the warmup cap: " + calls[0]);
        assertEquals(1, harness.results().size(), "one result");
        BenchResult result = harness.results().get(0);
        assertEquals("unsettled", result.name(), "name");
        assertTrue(Double.isFinite(result.meanNs()) && result.meanNs() >= 0.0, "mean");
        assertTrue(Double.isFinite(result.stddevNs()) && result.stddevNs() >= 0.0, "stddev");
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
    void accumulatesTheSinkAcrossEveryPhase() {
        BenchProfile profile = new BenchProfile(OptionalInt.of(3), OptionalInt.of(2), fixedWarmup(2, 1));
        BenchHarness harness = measured(profile).harness();

        harness.run("sink", 7, 5, () -> harness.sink().add(7));

        // 3 iterations over 2 warmup, 1 settling and 2 counted rounds.
        assertEquals(3 * (2 + 1 + 2) * 7, harness.sink().sum(), "sink sum");
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
