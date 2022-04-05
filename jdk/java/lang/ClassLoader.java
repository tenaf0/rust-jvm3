package java.lang;

public abstract class ClassLoader {
	native Class<?> loadClass(String name);

	protected Class<?> findClass(String name) throws Exception {
		throw new Exception(); // TODO: ClassNotFoundException
	}

	protected final native Class<?> defineClass(String name, byte[] b, int off, int len);
}
