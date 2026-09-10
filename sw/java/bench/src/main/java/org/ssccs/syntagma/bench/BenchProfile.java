package org.ssccs.syntagma.bench;

import java.util.Objects;
import java.util.OptionalInt;

/**
 * The iteration, round and warmup counts applied to every scenario.
 *
 * <p>The C++ suite in {@code sw/cpp/bench/bench.cpp} passes a fixed iteration
 * and round count to each scenario and relies on a single warmup call before
 * each timed round. Java needs executed code before a timed round means
 * anything, because the JIT compiles and profile-collects on the running
 * process rather than ahead of time, so this profile carries an explicit
 * warmup call count as well.
 *
 * <p>A present iteration or round override replaces the per-scenario count of
 * every scenario; an empty one keeps the count of the C++ suite, which is what
 * makes the defaults comparable with the C++ results. Below the profile, the
 * error-contract of the references applies: a non-positive count is a caller
 * error and raises {@link IllegalArgumentException}.
 *
 * @param iterations  the global iteration override, empty to keep the
 *                    per-scenario counts
 * @param rounds      the global round override, empty to keep the per-scenario
 *                    counts
 * @param warmupCalls the warmup calls run before the timed rounds, a Java
 *                    addition over the C++ harness
 */
public record BenchProfile(OptionalInt iterations, OptionalInt rounds, int warmupCalls) {

    public BenchProfile {
        Objects.requireNonNull(iterations, "iterations");
        Objects.requireNonNull(rounds, "rounds");
        if (iterations.isPresent() && iterations.getAsInt() < 1) {
            throw new IllegalArgumentException(
                    "BenchProfile: iterations must be at least 1, got " + iterations.getAsInt());
        }
        if (rounds.isPresent() && rounds.getAsInt() < 1) {
            throw new IllegalArgumentException(
                    "BenchProfile: rounds must be at least 1, got " + rounds.getAsInt());
        }
        if (warmupCalls < 0) {
            throw new IllegalArgumentException(
                    "BenchProfile: warmup calls must not be negative, got " + warmupCalls);
        }
    }

    /**
     * The default profile, comparable with the C++ suite: every scenario keeps
     * its own iteration and round count and pays three warmup calls.
     */
    public static BenchProfile standard() {
        return new BenchProfile(OptionalInt.empty(), OptionalInt.empty(), 3);
    }

    /**
     * The smoke profile of {@code --quick}: one iteration and one round per
     * scenario, with a single warmup call, so the suite finishes in seconds.
     * The workload of a single call is unchanged, so a scenario whose C++
     * count is one iteration still runs its full body.
     */
    public static BenchProfile quick() {
        return new BenchProfile(OptionalInt.of(1), OptionalInt.of(1), 1);
    }

    /**
     * This profile with the given overrides applied on top. An empty override
     * keeps the value of this profile, so {@code --iterations N} refines
     * {@code --quick} instead of replacing it.
     *
     * @throws IllegalArgumentException when an override is not positive
     */
    public BenchProfile withOverrides(OptionalInt iterationsOverride, OptionalInt roundsOverride) {
        Objects.requireNonNull(iterationsOverride, "iterationsOverride");
        Objects.requireNonNull(roundsOverride, "roundsOverride");
        return new BenchProfile(
                iterationsOverride.isPresent() ? iterationsOverride : iterations,
                roundsOverride.isPresent() ? roundsOverride : rounds,
                warmupCalls);
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
