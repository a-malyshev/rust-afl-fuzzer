Rust-AFL-Fuzzer is a simple mutation-based fuzzer which implements [AFL](https://lcamtuf.coredump.cx/afl/).


## How to run:

Once you built executable of the fuzzer with `cargo build` and testing program with `make build` (see one of the `examples`),
you can run the fuzzer over that compiled C program:

`./target/debug/fuzz -d=./examples/example1 -show example.c`

the option `-show` will print out useful statistics while the fuzzer is working.

`-d` points on the dirictory with testing program, its binary and meta information needed for analysis 
(e.g. to get feedback how many lines in the program have been executed)

the last option is the name of the C program which is going to be tested.

## or you can simply:

	1. Add permissions to execute buildscript.sh with this command:
		
		chmod +x buildscript.sh

	2. Run buildscript:

		./buildscript.sh

#### Under the hood, the buildscript will:

	- set up rust toolchain (rustc and cargo)
	- build fuzzer from *sources* and copy executable to the *test-app* (dir with testing C program)
	- build C program (example.c in the `example1` dir) with --coverage option
	- execute fuzzer over C program with option *-show* (which print out some insights while fuzzing)

