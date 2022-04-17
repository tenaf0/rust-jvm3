package java.lang;

import java.util.Objects;

public class StackTraceElement {
	private String declaringClass;
	private String methodName;
	private String fileName;
	private int lineNumber;

	public StackTraceElement(String declaringClass, String methodName,
			String fileName, int lineNumber) {
		this.declaringClass  = Objects.requireNonNull(declaringClass, "Declaring class is null");
		this.methodName      = Objects.requireNonNull(methodName, "Method name is null");
		this.fileName        = fileName;
		this.lineNumber      = lineNumber;
	}

	public String getClassName() {
		return declaringClass;
	}

	public String getMethodName() {
		return methodName;
	}
}