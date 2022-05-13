package hu.garaba;

public class Exception {
	public static void main(String[] args) {
		Pair[] arr = new Pair[20];
		for (int i = 0; i < arr.length; i++) {
			arr[i] = new Pair("A", i);
		}

		arr[10] = null;

		try {
			showArr(arr);
		} catch (NullPointerException e) {
			System.out.println("NPE");
		} catch (ArrayIndexOutOfBoundsException e) {
			System.out.println("AIOOBE");
		}

		try {
			showArr2(arr);
		} catch (NullPointerException e) {
			System.out.println("NPE");
		} catch (ArrayIndexOutOfBoundsException e) {
			System.out.println("AIOOBE");
		}
	}

	private static void showArr(Pair[] arr) {
		for (var pair : arr) {
			System.out.println(pair.a());
		}
	}

	private static void showArr2(Pair[] arr) {
		for (int i = 0; i < arr.length + 4; i++) {
			var pair = arr[i];
			try {
				System.out.println(pair.a());
			} catch (NullPointerException e) {
				System.out.println("null");
			}
		}
	}
}

final class Pair {
	private String a;
	private int b;

	public Pair(String a, int b) {
		this.a = a;
		this.b = b;
	}

	public String a() {
		return a;
	}
}