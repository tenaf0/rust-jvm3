package java.lang;

public final class Long {
	private final long value;

	public Long(long value) {
		this.value = value;
	}

	public static native long parseLong(String num);
}