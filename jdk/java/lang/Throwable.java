package java.lang;

public class Throwable {

	private StackTraceElement[] stackTrace;

	public Throwable() {
	}
	public Throwable(String message) {
		detailMessage = message;
	}

	public String getMessage() {
		return detailMessage;
	}

	private String detailMessage;
	
	public StackTraceElement[] getStackTrace() {
		return stackTrace; // TODO: Should be cloned
	}
}
