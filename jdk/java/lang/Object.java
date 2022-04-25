package java.lang;

// The compiled class file *doesn't* get loaded, it is only here as a reference.
// Consult the bootstrap.rs file for the actual runtime representation of the java/lang/Object class.
public class Object {
	public final native Class<?> getClass();
	public boolean equals(Object obj) {
		return (this == obj);
	}

	public native String toString();
}
