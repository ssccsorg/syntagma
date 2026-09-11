package org.ssccs.syntagma.bench;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;

import java.util.OptionalInt;

import org.junit.jupiter.api.Test;

/**
 * Cases for {@link BenchProfile}: the per-scenario counts of the C++ suite,
 * the collapse of {@code --quick}, and the override precedence the CLI
 * documents.
 */
class BenchProfileTest {

    @Test
    void standardKeepsTheCountsOfTheCppSuite() {
        BenchProfile profile = BenchProfile.standard();

        assertEquals(100000, profile.iterationsFor(100000), "iterations");
        assertEquals(3, profile.roundsFor(3), "rounds");
        assertEquals(BenchWarmup.steadyState(), profile.warmup(), "warmup policy");
    }

    @Test
    void quickCollapsesToOneIterationRoundAndWarmup() {
        BenchProfile profile = BenchProfile.quick();

        assertEquals(1, profile.iterationsFor(100000), "iterations");
        assertEquals(1, profile.roundsFor(5), "rounds");
        assertEquals(BenchWarmup.quick(), profile.warmup(), "warmup policy");
    }

    @Test
    void overridesReplaceTheCounts() {
        BenchProfile profile = BenchProfile.standard()
                .withOverrides(OptionalInt.of(7), OptionalInt.of(2));

        assertEquals(7, profile.iterationsFor(1), "iterations");
        assertEquals(2, profile.roundsFor(5), "rounds");
        assertEquals(BenchWarmup.steadyState(), profile.warmup(), "warmup policy is not overridable");
    }

    @Test
    void overridesRefineQuick() {
        BenchProfile profile = BenchProfile.quick()
                .withOverrides(OptionalInt.of(50), OptionalInt.empty());

        assertEquals(50, profile.iterationsFor(1), "iterations");
        assertEquals(1, profile.roundsFor(5), "rounds keep the quick value");
        assertEquals(BenchWarmup.quick(), profile.warmup(), "warmup policy keeps the quick value");
    }

    @Test
    void rejectsNonPositiveOverrides() {
        assertThrows(IllegalArgumentException.class,
                () -> new BenchProfile(OptionalInt.of(0), OptionalInt.empty(), BenchWarmup.quick()));
        assertThrows(IllegalArgumentException.class,
                () -> new BenchProfile(OptionalInt.empty(), OptionalInt.of(-1), BenchWarmup.quick()));
        assertThrows(NullPointerException.class,
                () -> new BenchProfile(OptionalInt.empty(), OptionalInt.empty(), null));
    }

    @Test
    void rejectsNonPositiveBaseCounts() {
        BenchProfile profile = BenchProfile.standard();

        assertThrows(IllegalArgumentException.class, () -> profile.iterationsFor(0));
        assertThrows(IllegalArgumentException.class, () -> profile.roundsFor(0));
    }
}
