package java.lang;

public final class Integer {
	private final int value;

	public Integer(int value) {
		this.value = value;
	}

	public static native int parseInt(String num);

	public static native String toString(int n);
}