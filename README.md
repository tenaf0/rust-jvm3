# Rust-JVM

A JVM made for educational purposes, implementing a subset of the specification.

## Build

Dependencies:
- rust + cargo
- OpenJDK 17

Issue the following command in the root of the project:
```
cargo build --release
```

And then go to the `jdk` folder and issue `make` to compile the `java.base` module and some example programs.

## Running

You can use the resulting executable as you would the `java` program:

```
./target/release/rust-jvm3 --cp jdk/target hu.garaba.Main [ARGS]
```