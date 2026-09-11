package org.ssccs.syntagma.bench;

import java.io.IOException;
import java.io.PrintStream;
import java.nio.file.Path;

/**
 * The entry point of the suite: the {@code main} of
 * {@code sw/cpp/bench/bench.cpp}, which {@code sw/cpp/run.sh --bench} drives as
 * {@code tagma_bench --json <result-dir>/bench-<timestamp>-<commit>.json
 * --commit <short-hash> --timestamp <timestamp>}.
 *
 * <p>The console report and the JSON summary keep the shape of the C++ suite:
 * a header line, the commit, one line per scenario with its name and its mean
 * nanoseconds per operation, the blackhole sink total, and the summary written
 * under {@code --json} when that flag is present. The scenario families
 * themselves are the ones of {@code sw/rust/benches/bench.rs}, the underlying
 * specification, recorded the same way by {@code sw/rust/benches/run.sh}.
 *
 * <p>Porting differences, all of them deliberate:
 *
 * <ul>
 *   <li>The exit status. The C++ program always returns zero, even when the
 *       result file could not be written. A usage error here returns 2 and a
 *       write failure returns 1, so a broken benchmark run cannot be mistaken
 *       for a successful one.</li>
 *   <li>{@link System#exit(int)} runs only for a non-zero status. The
 *       {@code exec:java} goal of the exec plugin starts the suite inside the
 *       Maven JVM, where an unconditional exit would kill Maven itself before
 *       it can print its build result.</li>
 * </ul>
 */
public final class BenchMain {

    private BenchMain() {
    }

    /**
     * Starts the suite and propagates a failure status to the process.
     *
     * @param args the command line; see {@link BenchCli}
     */
    public static void main(String[] args) {
        int status = run(args, System.out, System.err);
        if (status != 0) {
            System.exit(status);
        }
    }

    /**
     * Runs the suite and reports the status instead of exiting, so the entry
     * point stays callable from a test and from the Maven JVM.
     *
     * @param args the command line; see {@link BenchCli}
     * @param out  the console report stream
     * @param err  the error stream
     * @return 0 on success, 1 when the JSON summary could not be written, 2 on
     *         a usage error
     */
    public static int run(String[] args, PrintStream out, PrintStream err) {
        BenchCli.Options options;
        try {
            options = BenchCli.parse(args);
        } catch (IllegalArgumentException error) {
            err.println(BenchCli.PROGRAM + ": " + error.getMessage());
            BenchCli.printUsage(err);
            return 2;
        }
        if (options.help()) {
            BenchCli.printUsage(out);
            return 0;
        }
        BenchProfile profile = (options.quick() ? BenchProfile.quick() : BenchProfile.standard())
                .withOverrides(options.iterations(), options.rounds());

        out.println("=== Tagma Java Benchmark Suite ===");
        out.printf("commit: %s%n%n", options.commit());

        BenchHarness harness = new BenchHarness(profile, out);
        BenchSuite.runAll(harness);

        out.printf("%nsink: %d%n", harness.sink().sum());

        if (options.jsonPath().isPresent()) {
            Path path = options.jsonPath().orElseThrow();
            try {
                BenchJson.write(path, options.timestamp(), options.commit(), harness.results());
            } catch (IOException error) {
                err.println(BenchCli.PROGRAM + ": cannot write " + path + ": " + error.getMessage());
                return 1;
            }
            out.printf("Exported %d benchmarks to %s%n", harness.results().size(), path);
        }
        return 0;
    }
}
