package java.lang;

import java.io.PrintStream;

public final class System {
	public static final PrintStream out = null;

	private native static void registerNatives();
	static {
		registerNatives();
	}

	private System() {}
}