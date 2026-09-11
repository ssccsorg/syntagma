package org.ssccs.syntagma.bench;

import java.io.PrintStream;
import java.util.ArrayList;
import java.util.Collections;
import java.util.List;
import java.util.Objects;

/**
 * The steady-clock measurement harness: it warms a scenario up until two
 * consecutive rounds agree, drops the settling rounds, and reports the mean
 * nanoseconds per call with the standard deviation of the per-round means over
 * the counted rounds.
 *
 * <p>Port of the C++ {@code bench} function template in
 * {@code sw/cpp/bench/bench.cpp} together with its {@code g_sink} and
 * {@code g_results} globals, which Java carries as instance state because the
 * language has no free functions. The underlying scenario shape is the one of
 * {@code sw/rust/benches/bench.rs}.
 *
 * <p>A measurement runs in three phases, each of them on whole rounds of
 * {@code iterations} calls, the batch shape of the C++ harness:
 *
 * <ol>
 *   <li>The warmup phase keeps running rounds until two consecutive rounds
 *       agree within {@link BenchWarmup#tolerance()}, which is the steady-state
 *       criterion this port needs and the C++ harness does not: C++ is compiled
 *       ahead of time, so its single warmup call before each timed round
 *       suffices, while a Java scenario's first rounds are interpreted, then
 *       compiled, then recompiled under a better profile. The phase runs at
 *       least two rounds, because the first one has no predecessor to agree
 *       with, and agreement ends it once the warmup floor
 *       {@link BenchWarmup#minNanos()} is spent; it is cut off after
 *       {@link BenchWarmup#maxRounds()} rounds or
 *       {@link BenchWarmup#budgetNanos()}, so a scenario that never settles
 *       still terminates. The C++ per-round warmup call is subsumed by this
 *       phase and is not repeated, which saves one full body execution per
 *       round.</li>
 *   <li>The settling phase runs {@link BenchWarmup#settleRounds()} further
 *       rounds and discards them. That is the first timed round of the
 *       measurement budget, dropped so the clock interval that follows the
 *       warmup phase never enters the mean.</li>
 *   <li>The measurement phase times the counted rounds and records their mean
 *       and their population standard deviation.</li>
 * </ol>
 *
 * <p>A Java figure still carries a JIT-dependent component the C++ figure does
 * not. The compiler keeps profiling after the warmup phase, a deoptimization or
 * a later recompilation can move a counted round by a few percent, and the
 * platform is shared with the rest of the process, so a Java mean is an upper
 * bound on the steady-state cost where the ahead-of-time compiled C++ mean is
 * essentially flat. The reported standard deviation is where that residual
 * spread shows; two Java runs on the same machine can differ by a few percent
 * even when the scenario is stable, and the C++ suite is the reference for
 * absolute figures.
 *
 * <p>Two further porting differences follow from the language: the blackhole
 * sink is passed to the scenario through {@link #sink()} instead of a global
 * variable, so the two directions of the C++ free function namespace (harness
 * calls the closure, closure reads the sink) stay explicit; and the clock is
 * {@link System#nanoTime()}, the monotonic Java clock, in place of
 * {@code std::chrono::steady_clock}, which reports the same kind of elapsed
 * time free of wall-clock adjustments.
 *
 * <p>The harness is not thread-safe, mirroring the C++ process-wide globals.
 */
public final class BenchHarness {

    /**
     * The console line format of the C++ harness, {@code %-46s %12.1f ns/op
     * (stddev %10.1f)}.
     */
    private static final String LINE_FORMAT = "%-46s %12.1f ns/op  (stddev %10.1f)%n";

    private final BenchProfile profile;
    private final PrintStream out;
    private final Blackhole sink = new Blackhole();
    private final List<BenchResult> results = new ArrayList<>();

    /**
     * Creates a harness writing its console lines to {@code out}.
     *
     * @throws NullPointerException when {@code profile} or {@code out} is null
     */
    public BenchHarness(BenchProfile profile, PrintStream out) {
        this.profile = Objects.requireNonNull(profile, "profile");
        this.out = Objects.requireNonNull(out, "out");
    }

    /**
     * Measures one scenario and records its result.
     *
     * @param name           the scenario name, also the JSON key of the result
     * @param baseIterations the timed iteration count of the C++ suite
     * @param baseRounds     the round count of the C++ suite
     * @param op             one operation; its work must reach {@link #sink()},
     *                       otherwise the JIT is free to elide it
     * @throws IllegalArgumentException when a base count is not positive
     * @throws NullPointerException     when {@code name} or {@code op} is null
     */
    public void run(String name, int baseIterations, int baseRounds, Runnable op) {
        Objects.requireNonNull(name, "name");
        Objects.requireNonNull(op, "op");
        int iterations = profile.iterationsFor(baseIterations);
        int rounds = profile.roundsFor(baseRounds);
        BenchWarmup warmup = profile.warmup();

        warmUp(op, iterations, warmup);

        double[] perOpNs = new double[rounds];
        int timedRounds = warmup.settleRounds() + rounds;
        for (int r = 0; r < timedRounds; r++) {
            double perOp = (double) roundNanos(op, iterations) / iterations;
            if (r >= warmup.settleRounds()) {
                perOpNs[r - warmup.settleRounds()] = perOp;
            }
        }
        BenchResult result = BenchResult.statistics(name, perOpNs);
        results.add(result);
        out.printf(LINE_FORMAT, name, result.meanNs(), result.stddevNs());
    }

    /**
     * Runs whole warmup rounds until the floor {@link BenchWarmup#minNanos()}
     * is spent and two consecutive rounds agree, or until the round cap or the
     * warmup budget cuts the phase off. The first round never agrees, because it
     * has no predecessor, so a measurement is never taken from a completely
     * cold process.
     */
    private static void warmUp(Runnable op, int iterations, BenchWarmup warmup) {
        double previousNsPerOp = Double.NaN;
        long spentNanos = 0L;
        for (int round = 1; round <= warmup.maxRounds(); round++) {
            long elapsed = roundNanos(op, iterations);
            spentNanos += elapsed;
            double currentNsPerOp = (double) elapsed / iterations;
            if (spentNanos >= warmup.minNanos() && warmup.agrees(previousNsPerOp, currentNsPerOp)) {
                return;
            }
            if (spentNanos >= warmup.budgetNanos()) {
                return;
            }
            previousNsPerOp = currentNsPerOp;
        }
    }

    /** Times one round of {@code iterations} calls and returns its wall-clock nanoseconds. */
    private static long roundNanos(Runnable op, int iterations) {
        long start = System.nanoTime();
        for (int i = 0; i < iterations; i++) {
            op.run();
        }
        return System.nanoTime() - start;
    }

    /** The sink the scenarios accumulate their work into. */
    public Blackhole sink() {
        return sink;
    }

    /** The recorded results in execution order, as an unmodifiable view. */
    public List<BenchResult> results() {
        return Collections.unmodifiableList(results);
    }

    /**
     * An accumulating side effect the optimizer cannot elide: the C++
     * {@code g_sink} of {@code sw/cpp/bench/bench.cpp}, one {@code std::size_t}
     * widened to a Java {@code long}. The caller reports {@link #sum()} after
     * the timed loops, which is what keeps the measured work observable.
     */
    public static final class Blackhole {

        private long sum;

        /** Adds {@code value} to the accumulated sum. */
        public void add(long value) {
            sum += value;
        }

        /** The accumulated sum. */
        public long sum() {
            return sum;
        }
    }
}
