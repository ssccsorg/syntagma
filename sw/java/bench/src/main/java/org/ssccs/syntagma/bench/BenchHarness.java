package org.ssccs.syntagma.bench;

import java.io.PrintStream;
import java.util.ArrayList;
import java.util.Collections;
import java.util.List;
import java.util.Objects;

/**
 * The steady-clock measurement harness: it times a scenario closure
 * {@code iterations} times per round over {@code rounds} rounds and reports
 * the mean nanoseconds per call with the standard deviation of the per-round
 * means.
 *
 * <p>Port of the C++ {@code bench} function template in
 * {@code sw/cpp/bench/bench.cpp} together with its {@code g_sink} and
 * {@code g_results} globals, which Java carries as instance state because the
 * language has no free functions. The underlying scenario shape is the one of
 * {@code sw/rust/benches/bench.rs}.
 *
 * <p>Two porting differences follow from the language:
 *
 * <ul>
 *   <li>The explicit warmup phase. Before the timed rounds the harness runs
 *       the scenario {@link BenchProfile#warmupCalls()} times, because the JIT
 *       only compiles and optimizes code that has already executed; a timed
 *       round that pays for first-call compilation measures the compiler.
 *       C++ is compiled ahead of time, so the C++ harness needs the single
 *       warmup call it makes before each round and nothing more; that per-round
 *       call is kept here as well.</li>
 *   <li>The blackhole sink is passed to the scenario through
 *       {@link #sink()} instead of a global variable, so the two directions of
 *       the C++ free function namespace (harness calls the closure, closure
 *       reads the sink) stay explicit.</li>
 *   <li>The clock is {@link System#nanoTime()}, the monotonic Java clock, in
 *       place of {@code std::chrono::steady_clock}. Both report elapsed time
 *       free of wall-clock adjustments, so the nanosecond figures are the same
 *       kind of measurement.</li>
 * </ul>
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
        for (int i = 0; i < profile.warmupCalls(); i++) {
            op.run();
        }
        double[] perOpNs = new double[rounds];
        for (int r = 0; r < rounds; r++) {
            op.run(); // the single warmup call the C++ harness makes per round
            long start = System.nanoTime();
            for (int i = 0; i < iterations; i++) {
                op.run();
            }
            long end = System.nanoTime();
            perOpNs[r] = (double) (end - start) / iterations;
        }
        BenchResult result = BenchResult.statistics(name, perOpNs);
        results.add(result);
        out.printf(LINE_FORMAT, name, result.meanNs(), result.stddevNs());
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
