package hu.garaba.puzzlers;

public class LongDivision {

	public static void main(String[] args) {
		wrong();
		good();
	}
	private static void wrong() {
		final long MICROS_PER_DAY = 24 * 60 * 60 * 1000 * 1000;
		final long MILLIS_PER_DAY = 24 * 60 * 60 * 1000;

		System.out.println(MICROS_PER_DAY / MILLIS_PER_DAY);
	}

	private static void good() {
		final long MICROS_PER_DAY = 24L * 60 * 60 * 1000 * 1000;
		final long MILLIS_PER_DAY = 24L * 60 * 60 * 1000;

		System.out.println(MICROS_PER_DAY / MILLIS_PER_DAY);
	}
}
