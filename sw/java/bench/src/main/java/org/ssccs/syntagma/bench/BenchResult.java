package org.ssccs.syntagma.bench;

import java.util.Objects;

/**
 * One measured scenario: the mean nanoseconds per operation and the standard
 * deviation of the per-round means.
 *
 * <p>Mirrors the C++ {@code Result} struct of {@code sw/cpp/bench/bench.cpp};
 * the record component names map onto the JSON keys {@code mean_ns} and
 * {@code stddev_ns} written by the C++ {@code main}, and the underlying
 * scenario set is the one of {@code sw/rust/benches/bench.rs}.
 *
 * @param name      the scenario name, a JSON key in the summary
 * @param meanNs    mean nanoseconds per operation, averaged over the rounds
 * @param stddevNs  population standard deviation of the per-round means, in
 *                  nanoseconds
 */
public record BenchResult(String name, double meanNs, double stddevNs) {

    public BenchResult {
        Objects.requireNonNull(name, "name");
    }

    /**
     * Computes the statistics of {@code perOpNs}, the per-round means the
     * harness collects: the arithmetic mean over the rounds and the population
     * standard deviation over the same rounds, dividing the variance by the
     * round count exactly as the C++ harness does.
     *
     * @param name    the scenario name
     * @param perOpNs the per-round mean nanoseconds per operation
     * @throws IllegalArgumentException when {@code perOpNs} holds no round
     * @throws NullPointerException     when {@code perOpNs} is null
     */
    public static BenchResult statistics(String name, double[] perOpNs) {
        Objects.requireNonNull(perOpNs, "perOpNs");
        if (perOpNs.length == 0) {
            throw new IllegalArgumentException("BenchResult: at least one round is required");
        }
        double mean = 0.0;
        for (double value : perOpNs) {
            mean += value;
        }
        mean /= perOpNs.length;
        double variance = 0.0;
        for (double value : perOpNs) {
            variance += (value - mean) * (value - mean);
        }
        variance /= perOpNs.length;
        return new BenchResult(name, mean, Math.sqrt(variance));
    }
}
