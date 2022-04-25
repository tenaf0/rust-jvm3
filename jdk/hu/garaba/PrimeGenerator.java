package hu.garaba;

public class PrimeGenerator {
	private static int[] sieve;

	static {
		sieve = new int[1000000];

		init();
	}

	private static void init() {
		for (int i = 0; i < sieve.length; i++) {
			sieve[i] = i + 2;
		}
	}

	private static int findNum(int num) {
		return num - 2;
	}

	public static int calc() {
		for (int i = 0; i < sieve.length; i++) {
			var p = sieve[i];
			if (p == 0) {
				continue;
			}
			for (int j = findNum(p) + p; j < sieve.length; j += p) {
				sieve[j] = 0;
			}
		}

		var num = 0;
		for (int i = 0; i < sieve.length; i++) {
			if (sieve[i] == 0) {
				continue;
			}

			//out.println(sieve[i]);

			num++;
		}

		return num;
	}

	public static void main(String[] args) {
		for (int i = 0; i < 10; i++) {
			init();
			System.out.println(calc());
		}
	}
}