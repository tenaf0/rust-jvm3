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
	}
}

class A {
	void print() {
		System.out.println("A");
	}
}

class B extends A {
	void print() {
		System.out.println("B");
	}
}

class C extends A {
	void print() {
		System.out.println("C");
	}
}