package org.ssccs.syntagma.bench;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;

import org.junit.jupiter.api.Test;

/**
 * Cases for {@link BenchResult}: the mean and population standard deviation
 * computation of the C++ harness, on fixed inputs so the expected values are
 * exact.
 */
class BenchResultTest {

    @Test
    void computesTheMeanOverTheRounds() {
        BenchResult result = BenchResult.statistics("scenario", new double[] {1.0, 2.0, 3.0});

        assertEquals("scenario", result.name(), "name");
        assertEquals(2.0, result.meanNs(), 1e-12, "mean");
    }

    @Test
    void computesThePopulationStandardDeviation() {
        // Variance over three rounds is ((1-2)^2 + 0 + (3-2)^2) / 3 = 2/3,
        // divided by the round count as the C++ harness does.
        BenchResult result = BenchResult.statistics("scenario", new double[] {1.0, 2.0, 3.0});

        assertEquals(Math.sqrt(2.0 / 3.0), result.stddevNs(), 1e-12, "stddev");
    }

    @Test
    void aSingleRoundHasNoSpread() {
        BenchResult result = BenchResult.statistics("scenario", new double[] {12.0});

        assertEquals(12.0, result.meanNs(), 1e-12, "mean");
        assertEquals(0.0, result.stddevNs(), 1e-12, "stddev");
    }

    @Test
    void rejectsAnEmptyRoundSetAndANullName() {
        assertThrows(IllegalArgumentException.class, () -> BenchResult.statistics("scenario", new double[] {}));
        assertThrows(NullPointerException.class, () -> new BenchResult(null, 1.0, 0.0));
    }
}
