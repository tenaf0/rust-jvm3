package hu.garaba;

public class Main {
	public static void main(String[] args) {
		for (var s : args) {
			System.out.println(s);
		}
	}

	public static long gcd(long a, long b) {
		if (a < b) {
			return gcd(b, a);
		}

		if (b == 0) {
			return a;
		}
		return gcd(b, a % b);
	}

	private void step1(int[] arg) {
		for (int i = 0; i < 24; i++) {
			step2(arg, i);
		}
	}

	private void step2(int[] arg, int index) {
		System.out.println(arg[index]);
	}
}