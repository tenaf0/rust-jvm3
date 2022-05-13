package hu.garaba;

public class Inheritance {
	public static void main(String[] args) {
		A a = new A();
		A ab = new B();
		B b = new B();
		A ac = new C();
		C c = new C();

		a.print();
		ab.print();
		b.print();
		ac.print();
		c.print();

		System.out.println(a.data);
		System.out.println(ab.data);
		System.out.println(b.data);
		System.out.println(ac.data);
		System.out.println(c.data);
	}
}

class A {
	int data = 3;

	void print() {
		System.out.println("A");
	}
}

class B extends A {
	int data = 4;
	void print() {
		System.out.println("B");
	}
}

class C extends A {
	void print() {
		System.out.println("C");
	}
}