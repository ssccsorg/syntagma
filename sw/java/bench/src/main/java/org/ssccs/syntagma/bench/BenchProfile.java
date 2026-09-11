package org.ssccs.syntagma.bench;

import java.util.Objects;
import java.util.OptionalInt;

/**
 * The iteration, round and warmup configuration applied to every scenario.
 *
 * <p>The C++ suite in {@code sw/cpp/bench/bench.cpp} passes a fixed iteration
 * and round count to each scenario and relies on a single warmup call before
 * each timed round. Java needs more before a timed round means anything,
 * because the JIT compiles and profile-collects on the running process rather
 * than ahead of time, so the profile carries a {@link BenchWarmup} policy that
 * decides how long the harness warms up and how many settling rounds it drops.
 *
 * <p>A present iteration or round override replaces the per-scenario count of
 * every scenario; an empty one keeps the count of the C++ suite, which is what
 * makes the defaults comparable with the C++ results. Below the profile, the
 * error-contract of the references applies: a non-positive count is a caller
 * error and raises {@link IllegalArgumentException}.
 *
 * @param iterations the global iteration override, empty to keep the
 *                   per-scenario counts
 * @param rounds     the global round override, empty to keep the per-scenario
 *                   counts
 * @param warmup     the warmup policy of the harness, a Java addition over the
 *                   C++ harness
 */
public record BenchProfile(OptionalInt iterations, OptionalInt rounds, BenchWarmup warmup) {

    public BenchProfile {
        Objects.requireNonNull(iterations, "iterations");
        Objects.requireNonNull(rounds, "rounds");
        Objects.requireNonNull(warmup, "warmup");
        if (iterations.isPresent() && iterations.getAsInt() < 1) {
            throw new IllegalArgumentException(
                    "BenchProfile: iterations must be at least 1, got " + iterations.getAsInt());
        }
        if (rounds.isPresent() && rounds.getAsInt() < 1) {
            throw new IllegalArgumentException(
                    "BenchProfile: rounds must be at least 1, got " + rounds.getAsInt());
        }
    }

    /**
     * The default profile, comparable with the C++ suite: every scenario keeps
     * its own iteration and round count and warms up to steady state under
     * {@link BenchWarmup#steadyState()}.
     */
    public static BenchProfile standard() {
        return new BenchProfile(OptionalInt.empty(), OptionalInt.empty(), BenchWarmup.steadyState());
    }

    /**
     * The smoke profile of {@code --quick}: one iteration and one round per
     * scenario, with the single warmup round of {@link BenchWarmup#quick()}, so
     * the suite finishes in seconds. The workload of a single call is
     * unchanged, so a scenario whose C++ count is one iteration still runs its
     * full body.
     */
    public static BenchProfile quick() {
        return new BenchProfile(OptionalInt.of(1), OptionalInt.of(1), BenchWarmup.quick());
    }

    /**
     * This profile with the given overrides applied on top. An empty override
     * keeps the value of this profile, so {@code --iterations N} refines
     * {@code --quick} instead of replacing it, and the warmup policy is never
     * overridden by the command line.
     *
     * @throws IllegalArgumentException when an override is not positive
     */
    public BenchProfile withOverrides(OptionalInt iterationsOverride, OptionalInt roundsOverride) {
        Objects.requireNonNull(iterationsOverride, "iterationsOverride");
        Objects.requireNonNull(roundsOverride, "roundsOverride");
        return new BenchProfile(
                iterationsOverride.isPresent() ? iterationsOverride : iterations,
                roundsOverride.isPresent() ? roundsOverride : rounds,
                warmup);
    }

    /**
     * The timed iteration count of a scenario.
     *
     * @param base the count of the C++ suite
     * @throws IllegalArgumentException when {@code base} is not positive
     */
    public int iterationsFor(int base) {
        requirePositive(base, "base iterations");
        return iterations.isPresent() ? iterations.getAsInt() : base;
    }

    /**
     * The timed round count of a scenario.
     *
     * @param base the count of the C++ suite
     * @throws IllegalArgumentException when {@code base} is not positive
     */
    public int roundsFor(int base) {
        requirePositive(base, "base rounds");
        return rounds.isPresent() ? rounds.getAsInt() : base;
    }

    private static void requirePositive(int count, String what) {
        if (count < 1) {
            throw new IllegalArgumentException("BenchProfile: " + what + " must be at least 1, got " + count);
        }
    }
}
