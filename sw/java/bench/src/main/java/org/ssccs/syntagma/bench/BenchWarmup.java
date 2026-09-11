package org.ssccs.syntagma.bench;

/**
 * The warmup policy of the harness: it decides how long a scenario is run
 * before a number is recorded, and how many timed rounds are executed and
 * discarded before the counted ones.
 *
 * <p>The C++ suite in {@code sw/cpp/bench/bench.cpp} needs neither control: it
 * is compiled ahead of time, so its single warmup call before each timed round
 * is enough, and every timed round is recorded. Java measures a running,
 * profiling, compiling process, so the harness keeps running whole warmup
 * rounds until the phase is long enough and two consecutive rounds agree within
 * {@link #tolerance()}, and drops the settling rounds before it starts
 * counting.
 *
 * <p>Agreement alone is not a sufficient criterion, which is what
 * {@link #minNanos()} is for: a loop that is still interpreted can measure the
 * same slow time round after round, and then drop by an order of magnitude once
 * the compiler reaches it. Measured in this suite, the {@code space entry
 * or_insert 10k} body sits at about 1.0 ms per round for its first fifteen
 * rounds and then falls to about 0.024 ms, so a warmup that stops at the first
 * agreement reports a number the compiler had not touched yet. The floor makes
 * the phase outlast that kind of plateau.
 *
 * <p>The two caps keep the warmup bounded for a scenario that never settles,
 * either because its work is inherently variable or because the machine is
 * loaded: the round cap {@link #maxRounds()} and the wall-clock budget
 * {@link #budgetNanos()} of the whole warmup phase.
 *
 * @param minNanos     warmup wall-clock time that must be spent before
 *                     agreement may end the phase, at least 0
 * @param maxRounds    hard cap on warmup rounds, at least 1
 * @param tolerance    relative agreement of two consecutive warmup rounds, a
 *                     finite value of at least 0 where 0 requires identical
 *                     measurements and 1 accepts any two measurements
 * @param budgetNanos  hard wall-clock cap of the whole warmup phase in
 *                     nanoseconds, at least {@code minNanos}
 * @param settleRounds timed rounds executed and discarded before the counted
 *                     rounds, at least 0; this is the first timed round of the
 *                     measurement budget
 */
public record BenchWarmup(long minNanos, int maxRounds, double tolerance, long budgetNanos, int settleRounds) {

    public BenchWarmup {
        if (minNanos < 0L) {
            throw new IllegalArgumentException("BenchWarmup: minNanos must not be negative, got " + minNanos);
        }
        if (maxRounds < 1) {
            throw new IllegalArgumentException("BenchWarmup: maxRounds must be at least 1, got " + maxRounds);
        }
        if (!Double.isFinite(tolerance) || tolerance < 0.0) {
            throw new IllegalArgumentException(
                    "BenchWarmup: tolerance must be finite and not negative, got " + tolerance);
        }
        if (budgetNanos < minNanos) {
            throw new IllegalArgumentException(
                    "BenchWarmup: budgetNanos " + budgetNanos + " must not be below minNanos " + minNanos);
        }
        if (settleRounds < 0) {
            throw new IllegalArgumentException(
                    "BenchWarmup: settleRounds must not be negative, got " + settleRounds);
        }
    }

    /**
     * The default policy, used whenever {@code --quick} is absent: a warmup
     * floor of one second with agreement within five percent of the previous
     * round, bounded by three seconds or four thousand rounds, and two settling
     * rounds discarded.
     *
     * <p>The floor is far above the worst plateau observed in this suite, about
     * fifteen rounds at one millisecond apiece, which is where the compiler
     * reaches a ten-thousand-iteration loop body; the budget bounds a scenario
     * whose rounds never agree, and the round cap bounds one whose rounds are
     * too cheap to spend the budget at all.
     */
    public static BenchWarmup steadyState() {
        return new BenchWarmup(1_000_000_000L, 4000, 0.05, 3_000_000_000L, 2);
    }

    /**
     * The smoke policy of {@code --quick}: exactly one warmup round, no floor
     * and no budget beyond it, and no settling round, so a scenario is executed
     * twice in total. The reported figure of a quick run is a cold-start figure
     * by construction, which is what keeps the smoke run in the seconds range.
     */
    public static BenchWarmup quick() {
        return new BenchWarmup(0L, 1, 0.10, 0L, 0);
    }

    /**
     * Whether two consecutive warmup rounds agree, which, together with a spent
     * warmup floor, ends the warmup phase.
     *
     * <p>Agreement is relative to the larger of the two measurements, so the
     * test means the same thing for a scenario in the hundreds of nanoseconds
     * and for one in the tens of milliseconds. Two rounds that both measured
     * zero agree trivially; a measurement that is not finite never agrees,
     * because an unstable round is exactly the case the warmup phase exists
     * for, and the missing previous round of the first warmup round is not
     * finite either, which is what keeps the phase at two rounds minimum.
     *
     * @param previousNsPerOp the mean nanoseconds per operation of the previous
     *                        warmup round, not a number before the first one
     * @param currentNsPerOp  the mean nanoseconds per operation of the round
     *                        that just finished
     */
    public boolean agrees(double previousNsPerOp, double currentNsPerOp) {
        if (!Double.isFinite(previousNsPerOp) || !Double.isFinite(currentNsPerOp)) {
            return false;
        }
        double larger = Math.max(Math.abs(previousNsPerOp), Math.abs(currentNsPerOp));
        if (larger == 0.0) {
            return true;
        }
        return Math.abs(currentNsPerOp - previousNsPerOp) <= tolerance * larger;
    }
}
