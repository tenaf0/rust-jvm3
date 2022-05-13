package java.lang;
public final class String {
	long length;
	long index;

	public native String concat(String other);

	public long length() {
		return length;
	}

	public native char charAt(int index);

	public boolean equals(Object o) {
		return StringUtil.stringEquals(this, o);
	}
}