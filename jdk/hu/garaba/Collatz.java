package hu.garaba;

public class Collatz {

	public static void main(String[] args) {
		long fuel = Long.parseLong(args[0]);

		long start = 1;
		CollatzStep coll = Continue.startFrom(start);

		for (long stepCount = 0; stepCount < fuel; stepCount++) {
			if (coll instanceof Continue c) {
				coll = c.next();
			} else {
				var f = (Finished) coll;
				System.out.print(f.start);
				System.out.print(" ");
				System.out.println(f.steps);

				start++;
				coll = Continue.startFrom(start);
			}
		}
	}
}

sealed interface CollatzStep permits Continue, Finished { }

final class Continue implements CollatzStep {
	long start;
	long current;
	long steps;

	public Continue(long start, long current, long steps) {
		this.start = start;
		this.current = current;
		this.steps = steps;
	}

	public static Continue startFrom(long start) {
		return new Continue(start, start, 0);
	}

	public CollatzStep next() {
		long next;
		if (current % 2 == 0) {
			next = current / 2;
		} else {
			next = 3 * current + 1;
		}

		if (next == 1) {
			return new Finished(start, steps + 1);
		} else {
			return new Continue(start, next, steps + 1);
		}
	}
}

final class Finished implements CollatzStep {
	long start;
	long steps;
	public Finished(long start, long steps) {
		this.start = start;
		this.steps = steps;
	}
}