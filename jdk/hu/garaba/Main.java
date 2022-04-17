package hu.garaba;

public class Main {
	public static void main(String[] args) {
		var main = new Main();
		//try {
			int[] arg = new int[10];
			main.step1(arg);
		//} catch (Exception e) {
		//	for (var elem : e.getStackTrace()) {
		//		System.out.println(elem.getClassName());
		//		System.out.println(elem.getMethodName());
		//		System.out.println("");
		//	}
		//}
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