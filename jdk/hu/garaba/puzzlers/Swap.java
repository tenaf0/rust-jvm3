package hu.garaba.puzzlers;

public class Swap {
	public static void main(String[] args) {
		int x = 1984; // (0x7c0)
		int y = 2001; // (0x7d1)
		x ^= y ^= x ^= y;
		System.out.println("x = ".concat(Integer.toString(x)).concat("; y = ").concat(Integer.toString(y)));
	}
}