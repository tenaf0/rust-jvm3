package hu.garaba.puzzlers;

public class AnimalFarm {
	public static void main(String[] args) {
		final String pig = "length: 10";
		final String dog = "length: ".concat(Integer.toString((int) pig.length()));
		System.out.println("Animals are equal: ".concat(Boolean.toString(pig == dog)));
	}
}
