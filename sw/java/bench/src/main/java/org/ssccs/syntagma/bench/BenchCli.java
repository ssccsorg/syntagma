package org.ssccs.syntagma.bench;

import java.io.PrintStream;
import java.nio.file.InvalidPathException;
import java.nio.file.Path;
import java.util.Objects;
import java.util.Optional;
import java.util.OptionalInt;

/**
 * The command line of the suite: {@code --json PATH}, {@code --commit SHA} and
 * {@code --timestamp TS} are the contract of the C++ {@code main} in
 * {@code sw/cpp/bench/bench.cpp}, which {@code sw/cpp/run.sh --bench} drives;
 * {@code --iterations N}, {@code --rounds N} and {@code --quick} are the Java
 * smoke-run additions the C++ suite has no counterpart for, because it hardcodes
 * the counts per scenario.
 *
 * <p>Porting differences, all of them deliberate:
 *
 * <ul>
 *   <li>A flag whose value is missing or malformed is rejected with
 *       {@link IllegalArgumentException}, where the C++ loop silently ignores
 *       the flag. A silently dropped {@code --json} would make a benchmark run
 *       look successful while writing no result file.</li>
 *   <li>Unknown arguments are ignored, exactly as in the C++ loop.</li>
 *   <li>The usage text names the program {@code tagma-bench} rather than
 *       {@code argv[0]}, because the suite also starts through
 *       {@code mvn exec:java}, where no argument vector entry carries the
 *       command name.</li>
 * </ul>
 */
public final class BenchCli {

    /** The program name shown in the usage text. */
    public static final String PROGRAM = "tagma-bench";

    /** The commit and timestamp recorded when the caller passes none. */
    public static final String UNKNOWN = "unknown";

    private BenchCli() {
    }

    /**
     * The parsed command line.
     *
     * @param jsonPath   where the JSON summary is written, empty when the run
     *                   reports to the console only
     * @param commit     the commit recorded in the summary
     * @param timestamp  the timestamp recorded in the summary
     * @param iterations the global iteration override of {@code --iterations}
     * @param rounds     the global round override of {@code --rounds}
     * @param quick      whether {@code --quick} asked for the smoke profile
     * @param help       whether {@code --help} asked for the usage text
     */
    public record Options(Optional<Path> jsonPath, String commit, String timestamp,
            OptionalInt iterations, OptionalInt rounds, boolean quick, boolean help) {

        public Options {
            Objects.requireNonNull(jsonPath, "jsonPath");
            Objects.requireNonNull(commit, "commit");
            Objects.requireNonNull(timestamp, "timestamp");
            Objects.requireNonNull(iterations, "iterations");
            Objects.requireNonNull(rounds, "rounds");
        }
    }

    /**
     * Parses the argument vector from left to right, matching the loop of the
     * C++ {@code main} and the flags added on top of it.
     *
     * @throws IllegalArgumentException when a flag value is missing, is not an
     *                                  integer, is not positive, or is not a
     *                                  usable path
     * @throws NullPointerException     when {@code args} is null
     */
    public static Options parse(String[] args) {
        Objects.requireNonNull(args, "args");
        Path jsonPath = null;
        String commit = UNKNOWN;
        String timestamp = UNKNOWN;
        OptionalInt iterations = OptionalInt.empty();
        OptionalInt rounds = OptionalInt.empty();
        boolean quick = false;
        boolean help = false;
        for (int i = 0; i < args.length; i++) {
            String argument = args[i];
            if (argument.equals("--json")) {
                jsonPath = path(value(args, i, "--json"));
                i += 1;
            } else if (argument.equals("--commit")) {
                commit = value(args, i, "--commit");
                i += 1;
            } else if (argument.equals("--timestamp")) {
                timestamp = value(args, i, "--timestamp");
                i += 1;
            } else if (argument.equals("--iterations")) {
                iterations = OptionalInt.of(positive(args, i, "--iterations"));
                i += 1;
            } else if (argument.equals("--rounds")) {
                rounds = OptionalInt.of(positive(args, i, "--rounds"));
                i += 1;
            } else if (argument.equals("--quick")) {
                quick = true;
            } else if (argument.equals("--help")) {
                help = true;
            }
            // Any other argument is ignored, mirroring the C++ argument loop.
        }
        return new Options(Optional.ofNullable(jsonPath), commit, timestamp, iterations, rounds, quick,
                help);
    }

    /** Prints the usage text, mirroring {@code print_usage} of the C++ suite. */
    public static void printUsage(PrintStream out) {
        Objects.requireNonNull(out, "out");
        out.printf("Usage: %s [--json PATH] [--commit SHA] [--timestamp TS]%n"
                + "       %s [--iterations N] [--rounds N] [--quick] [--help]%n"
                + "  --json PATH      write a JSON summary of the results%n"
                + "  --commit SHA     commit identifier recorded in the JSON%n"
                + "  --timestamp TS   timestamp recorded in the JSON%n"
                + "  --iterations N   timed iterations per round in every scenario%n"
                + "  --rounds N       timed rounds in every scenario%n"
                + "  --quick          smoke profile: one iteration, one round, one warmup call%n"
                + "  --help           print this usage%n",
                PROGRAM, PROGRAM);
    }

    private static String value(String[] args, int index, String flag) {
        if (index + 1 >= args.length) {
            throw new IllegalArgumentException(flag + " requires a value");
        }
        return args[index + 1];
    }

    private static int positive(String[] args, int index, String flag) {
        String raw = value(args, index, flag);
        int parsed;
        try {
            parsed = Integer.parseInt(raw);
        } catch (NumberFormatException error) {
            throw new IllegalArgumentException(flag + " requires an integer, got " + raw, error);
        }
        if (parsed < 1) {
            throw new IllegalArgumentException(flag + " requires a positive integer, got " + raw);
        }
        return parsed;
    }

    private static Path path(String raw) {
        try {
            return Path.of(raw);
        } catch (InvalidPathException error) {
            throw new IllegalArgumentException("--json is not a usable path: " + raw, error);
        }
    }
}
