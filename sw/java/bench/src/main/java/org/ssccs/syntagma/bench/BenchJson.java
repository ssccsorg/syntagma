package org.ssccs.syntagma.bench;

import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.List;
import java.util.Objects;

/**
 * The JSON summary writer: the object the C++ {@code main} streams to
 * {@code --json PATH} from the block at the end of
 * {@code sw/cpp/bench/bench.cpp}.
 *
 * <p>The schema is the one of the references and of
 * {@code sw/rust/benches/export_results.py}, three members at the top level and
 * one member per scenario:
 *
 * <pre>{@code
 * {
 *   "timestamp": "...",
 *   "commit": "...",
 *   "benchmarks": {
 *     "scenario": {"mean_ns": <double>, "stddev_ns": <double>}
 *   }
 * }
 * }</pre>
 *
 * <p>Numbers are written with {@link Double#toString(double)}, the shortest
 * decimal that round-trips the value, where the C++ stream writes six
 * significant digits. Both are valid JSON numbers, and the Java form keeps the
 * full precision of the measurement. Two guards are added, because a JSON
 * document cannot carry them: a non-finite measurement is rejected, and a
 * quote, backslash or control character in the name, commit or timestamp is
 * escaped rather than emitted verbatim.
 */
public final class BenchJson {

    private BenchJson() {
    }

    /**
     * Renders the summary document.
     *
     * @param timestamp the timestamp recorded under {@code "timestamp"}
     * @param commit    the commit recorded under {@code "commit"}
     * @param results   the measured scenarios, in execution order
     * @throws IllegalArgumentException when a measurement is not finite
     * @throws NullPointerException     when an argument is null
     */
    public static String render(String timestamp, String commit, List<BenchResult> results) {
        Objects.requireNonNull(timestamp, "timestamp");
        Objects.requireNonNull(commit, "commit");
        Objects.requireNonNull(results, "results");
        StringBuilder out = new StringBuilder();
        out.append("{\n");
        out.append("  \"timestamp\": \"").append(escape(timestamp)).append("\",\n");
        out.append("  \"commit\": \"").append(escape(commit)).append("\",\n");
        out.append("  \"benchmarks\": {\n");
        for (int i = 0; i < results.size(); i++) {
            BenchResult result = results.get(i);
            out.append("    \"").append(escape(result.name()))
                    .append("\": {\"mean_ns\": ").append(number(result.meanNs()))
                    .append(", \"stddev_ns\": ").append(number(result.stddevNs()))
                    .append('}');
            out.append(i + 1 < results.size() ? ",\n" : "\n");
        }
        out.append("  }\n}\n");
        return out.toString();
    }

    /**
     * Writes the summary document to {@code path}.
     *
     * @throws IOException          when the file cannot be written
     * @throws IllegalArgumentException when a measurement is not finite
     * @throws NullPointerException when an argument is null
     */
    public static void write(Path path, String timestamp, String commit, List<BenchResult> results)
            throws IOException {
        Objects.requireNonNull(path, "path");
        Files.writeString(path, render(timestamp, commit, results), StandardCharsets.UTF_8);
    }

    private static String number(double value) {
        if (!Double.isFinite(value)) {
            throw new IllegalArgumentException("BenchJson: the measurement is not finite: " + value);
        }
        return Double.toString(value);
    }

    /**
     * Escapes the characters a JSON string cannot carry literally. The C++
     * writer emits the names verbatim, which holds for the suite's own names
     * but not for a commit or timestamp a caller may pass in.
     */
    private static String escape(String text) {
        StringBuilder out = new StringBuilder(text.length());
        for (int i = 0; i < text.length(); i++) {
            char ch = text.charAt(i);
            switch (ch) {
                case '"' -> out.append("\\\"");
                case '\\' -> out.append("\\\\");
                case '\n' -> out.append("\\n");
                case '\r' -> out.append("\\r");
                case '\t' -> out.append("\\t");
                default -> {
                    if (ch < 0x20) {
                        out.append(String.format("\\u%04x", (int) ch));
                    } else {
                        out.append(ch);
                    }
                }
            }
        }
        return out.toString();
    }
}
