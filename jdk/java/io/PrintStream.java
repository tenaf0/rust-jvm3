package java.io;

public class PrintStream {
	public native void print(char x);
	public native void print(int x);
	public native void print(long x);
	public native void print(double x);
	public native void print(String x);

	public void println(int x) {
		print(x);
		print('\n');
	}

	public void println(long x) {
		print(x);
		print('\n');
	}

	public void println(double x) {
		print(x);
		print('\n');
	}

	public void println(String x) {
		print(x);
		print('\n');
	}
}