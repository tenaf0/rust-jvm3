package java.lang;

public class Boolean {
	private final boolean value;

	public Boolean(boolean value) {
		this.value = value;
	}

	public static String toString(boolean bool) {
		return bool ? "true" : "false";
	}
}