package java.util;

public final class Objects {
	private Objects() {
		throw new Error();
	}

	public static <T> T requireNonNull(T obj, String message) {
		if (obj == null)
			throw new NullPointerException(message);
		return obj;
	}
}