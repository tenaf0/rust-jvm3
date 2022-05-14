package hu.garaba;

public class Main {
	public static void main(String[] args) {
		for (int i = 0; i < 20; i++) {
			PrimeGenerator.main(new String[] {});
			nbody.main(new String[] { "300000" });
			BinaryTree.main(new String[] { "14" });
			Collatz.main(new String[] { "2000000" });
		}
	}
}
