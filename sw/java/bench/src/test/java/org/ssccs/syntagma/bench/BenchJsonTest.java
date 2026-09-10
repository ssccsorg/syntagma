package org.ssccs.syntagma.bench;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.List;

import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;

/**
 * Cases for {@link BenchJson}: the exact schema the C++ {@code main} of
 * {@code sw/cpp/bench/bench.cpp} writes, the escaping of text that a JSON
 * string cannot carry verbatim, and the file entry point.
 */
class BenchJsonTest {

    @Test
    void rendersTheSchemaOfTheCppWriter() {
        String json = BenchJson.render("20260101-000000", "abc1234", List.of(
                new BenchResult("csn2 insert all 10k", 12.5, 0.25),
                new BenchResult("map static single insert", 1.0, 0.0)));

        String expected = String.join("\n",
                "{",
                "  \"timestamp\": \"20260101-000000\",",
                "  \"commit\": \"abc1234\",",
                "  \"benchmarks\": {",
                "    \"csn2 insert all 10k\": {\"mean_ns\": 12.5, \"stddev_ns\": 0.25},",
                "    \"map static single insert\": {\"mean_ns\": 1.0, \"stddev_ns\": 0.0}",
                "  }",
                "}",
                "");
        assertEquals(expected, json, "json summary");
    }

    @Test
    void anEmptyResultSetKeepsTheSameShape() {
        String json = BenchJson.render("ts", "commit", List.of());

        String expected = String.join("\n",
                "{",
                "  \"timestamp\": \"ts\",",
                "  \"commit\": \"commit\",",
                "  \"benchmarks\": {",
                "  }",
                "}",
                "");
        assertEquals(expected, json, "json summary");
    }

    @Test
    void escapesTextAJsonStringCannotCarry() {
        String json = BenchJson.render("tab\tend", "sha\"quoted\"\\slash", List.of(
                new BenchResult("name\u0001with control", 1.0, 0.0)));

        assertTrue(json.contains("\"timestamp\": \"tab\\tend\""), json);
        assertTrue(json.contains("\"commit\": \"sha\\\"quoted\\\"\\\\slash\""), json);
        assertTrue(json.contains("\"name\\u0001with control\""), json);
        assertTrue(json.startsWith("{\n  \"timestamp\""), json);
    }

    @Test
    void rejectsAMeasurementThatJsonCannotCarry() {
        assertThrows(IllegalArgumentException.class, () -> BenchJson.render("ts", "commit",
                List.of(new BenchResult("scenario", Double.NaN, 0.0))));
    }

    @Test
    void writesTheRenderedDocument(@TempDir Path directory) throws IOException {
        Path target = directory.resolve("bench-20260101-000000-abc1234.json");

        BenchJson.write(target, "20260101-000000", "abc1234",
                List.of(new BenchResult("scenario", 3.5, 0.5)));

        assertEquals(BenchJson.render("20260101-000000", "abc1234",
                List.of(new BenchResult("scenario", 3.5, 0.5))),
                Files.readString(target, StandardCharsets.UTF_8), "written document");
    }

    @Test
    void reportsAnUnwritablePath(@TempDir Path directory) {
        Path missingDirectory = directory.resolve("absent").resolve("bench.json");

        assertThrows(IOException.class, () -> BenchJson.write(missingDirectory, "ts", "commit", List.of()));
    }
}
