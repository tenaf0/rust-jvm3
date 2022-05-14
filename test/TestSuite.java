package test;

import java.io.IOException;
import java.io.BufferedReader;
import java.io.InputStreamReader;
import java.util.List;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.stream.Collectors;

public final class TestSuite {


	public static void main(String[] args) {
		final boolean test = false;

		if (test) {
			integratationTest();
		} else {
			long start = 80000;
			long step = 140000;

			var times = new HashMap<Long, List<Double>>();

			int i = 0;
			final int stepNo = 20;
			for (; i < stepNo; i++) {
				System.out.printf("%d/%d\n\n", i, stepNo);
				var result = benchmark(7, new Test("Collatz", List.of("hu.garaba.Collatz", "10000")),
						new Test("Collatz", List.of("hu.garaba.Collatz", Long.toString(start))));
				times.put(start, result);
				start += step;
			}

			for (var entry : times.entrySet()) {
				System.out.printf("%d, %f, %f, %f\n", entry.getKey(),
						entry.getValue().get(0), entry.getValue().get(1), entry.getValue().get(2));
			}
		}
	}
	private static List<Double> benchmark(int n, Test warmupTest, Test test) {
		final int warmup = 4;

		var runners = new ArrayList<TestRunner>();
		runners.add(new CmdRunner(List.of("target/release/rust-jvm3", "--cp", "jdk/target", "--"), true));
		runners.add(new CmdRunner(List.of("java", "-cp", "jdk/target_JDK"), true));
		runners.add(new CmdRunner(List.of("java", "-cp", "jdk/target_JDK", "-Djava.compiler=NONE"), true));

		var times = new ArrayList<Double>();

		for (var runner : runners) {
			System.out.println(runner + " warming up");

			for (int i = 0; i < warmup; i++) {
				runner.run(warmupTest);
			}

			System.out.println("benchmark started");

			long time = 0;
			for (int i = 0; i < n; i++) {
				var result = runner.run(test);
				time += result.time();
			}

			double avg = time / 1e9 / n;
			times.add(avg);
			System.out.println("Finished execution on average in " + avg + "s");
		}

		return times;
	}
	private static void integratationTest() {
		var runners = new ArrayList<TestRunner>();
		runners.add(new CmdRunner(List.of("target/release/rust-jvm3", "--cp", "jdk/target", "--")));
		runners.add(new CmdRunner(List.of("java", "-cp", "jdk/target_JDK")));

		var tests = new ArrayList<Test>();
		tests.add(new Test("Prime test", List.of("hu.garaba.PrimeGenerator")));
		tests.add(new Test("nbody", List.of("hu.garaba.nbody", "100000")));
		tests.add(new Test("Inheritance", List.of("hu.garaba.Inheritance")));
		tests.add(new Test("Collatz", List.of("hu.garaba.Collatz", "10000")));
		tests.add(new Test("Exception", List.of("hu.garaba.Exception")));
		tests.add(new Test("BinaryTree", List.of("hu.garaba.BinaryTree", "10")));
		tests.add(new Test("AnimalFarm", List.of("hu.garaba.puzzlers.AnimalFarm")));
		tests.add(new Test("DosEquis", List.of("hu.garaba.puzzlers.DosEquis")));
		tests.add(new Test("LongDivision", List.of("hu.garaba.puzzlers.LongDivision")));
		tests.add(new Test("Multicast", List.of("hu.garaba.puzzlers.Multicast", "-1")));
		tests.add(new Test("Multicast 2", List.of("hu.garaba.puzzlers.Multicast", "-255")));
		tests.add(new Test("Swap", List.of("hu.garaba.puzzlers.Swap")));

		var successfulTests = 0;

		for (var test : tests) {
			System.out.println("#####\nStarted running " + test.name() + "\n#####\n");

			var results = new ArrayList<TestResult>();

			for (var runner : runners) {
				var res = runner.run(test);
				results.add(res);
				System.out.printf("\nElapsed time: %.2fs\n###\n\n", (res.time() / 1e9));
			}

			if (results.stream().allMatch(s -> {
				return s.success() && s.stdout().equals(results.get(0).stdout());
			})) {
				System.out.println("Successful test");
				successfulTests += 1;
			} else {
				System.out.println("Output of tests didn't match");
				for (var output : results) {
					System.out.println(output);
				}
			}

			System.out.println("\n##########\n\n");
		}

		System.out.printf("%d test(s) out of %d succeeded\n\n", successfulTests, tests.size());
	}
}

record Test(String name, List<String> args) {}

record TestResult(boolean success, long time, String stdout, String stderr) {}

interface TestRunner {
	TestResult run(Test test);
}

final record CmdRunner(List<String> commands, boolean silent) implements TestRunner {
	public CmdRunner(List<String> commands) {
		this(commands, false);
	}

	public TestResult run(Test test) {
		var command = new ArrayList<String>();
		command.addAll(commands);
		command.addAll(test.args());

		var pb = new ProcessBuilder(command);
		var now = System.nanoTime();
		try {
			var process = pb.start();
			var inputStream = process.getInputStream();
			var errorStream = process.getErrorStream();
			try (var inputReader = new BufferedReader(new InputStreamReader(inputStream));
					var errorReader = new BufferedReader(new InputStreamReader(errorStream))) {

				String result = inputReader.lines().peek(s -> {
					if (!silent)
						System.out.println(s);
				}).collect(Collectors.joining(
						"\n"));
				String error = errorReader.lines().collect(Collectors.joining("\n"));
				return new TestResult(true, System.nanoTime() - now, result, error);
			}
		} catch (IOException e) {
			System.err.println("Exception happened: " +  e);
		}

		return new TestResult(false, System.nanoTime() - now, null, "Exception occured");
	}
}
