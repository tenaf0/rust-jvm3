package hu.garaba;

public class Main {
	public static void main(String[] args) {
		for (int i = 0; i < 20; i++) {
			System.out.println("Running PrimeGenerator");
			PrimeGenerator.main(args);

			System.out.println("Running nbody");
			nbody.main(args);

			System.out.println("Running Virtual");
			for (int j = 0; j < 20; j++) {
				Virtual.main(args);
			}
		}
	}
}