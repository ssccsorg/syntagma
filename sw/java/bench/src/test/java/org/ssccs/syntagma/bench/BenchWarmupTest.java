package org.ssccs.syntagma.bench;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import org.junit.jupiter.api.Test;

/**
 * Cases for {@link BenchWarmup}: the two policies the profiles use, the
 * agreement rule that ends the warmup phase, and the rejections that keep a
 * policy self-consistent.
 */
class BenchWarmupTest {

    @Test
    void steadyStateIsBoundedAndDropsOneSettlingRound() {
        BenchWarmup warmup = BenchWarmup.steadyState();

        assertTrue(warmup.minNanos() > 0L, "a warmup floor above zero");
        assertTrue(warmup.maxRounds() > 1, "a cap above the first round");
        assertTrue(warmup.tolerance() > 0.0, "a positive tolerance");
        assertTrue(warmup.budgetNanos() >= warmup.minNanos(), "a finite budget");
        assertTrue(warmup.settleRounds() >= 1, "the first timed round is discarded");
        assertFalse(warmup.agrees(Double.NaN, 1.0),
                "the first warmup round never agrees, so at least two run");
    }

    @Test
    void quickRunsOneWarmupRoundAndDiscardsNothing() {
        BenchWarmup warmup = BenchWarmup.quick();

        assertTrue(warmup.maxRounds() == 1, "one warmup round");
        assertTrue(warmup.settleRounds() == 0, "no settling round");
    }

    @Test
    void agreesWithinTheRelativeTolerance() {
        BenchWarmup warmup = BenchWarmup.steadyState();

        assertTrue(warmup.agrees(100.0, 105.0), "five percent apart");
        assertTrue(warmup.agrees(105.0, 100.0), "agreement is symmetric");
        assertFalse(warmup.agrees(100.0, 130.0), "thirty percent apart");
    }

    @Test
    void twoZeroRoundsAgreeAndANonFiniteRoundNeverDoes() {
        BenchWarmup warmup = BenchWarmup.steadyState();

        assertTrue(warmup.agrees(0.0, 0.0), "nothing measured twice");
        assertFalse(warmup.agrees(Double.NaN, 1.0), "a missing previous round");
        assertFalse(warmup.agrees(1.0, Double.NaN), "a missing current round");
        assertFalse(warmup.agrees(1.0, Double.POSITIVE_INFINITY), "an overflowed current round");
    }

    @Test
    void rejectsAnInconsistentPolicy() {
        assertThrows(IllegalArgumentException.class, () -> new BenchWarmup(-1, 1, 0.1, 0L, 0));
        assertThrows(IllegalArgumentException.class, () -> new BenchWarmup(1, 0, 0.1, 0L, 0));
        assertThrows(IllegalArgumentException.class, () -> new BenchWarmup(3, 2, 0.1, 0L, 0));
        assertThrows(IllegalArgumentException.class, () -> new BenchWarmup(1, 1, -0.1, 0L, 0));
        assertThrows(IllegalArgumentException.class, () -> new BenchWarmup(1, 1, Double.NaN, 0L, 0));
        assertThrows(IllegalArgumentException.class, () -> new BenchWarmup(1, 1, 0.1, -1L, 0));
        assertThrows(IllegalArgumentException.class, () -> new BenchWarmup(1, 1, 0.1, 0L, -1));
    }
}
