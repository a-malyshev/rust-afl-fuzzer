Fuzz is a simple mutation-based fuzzer which implements [AFL](https://lcamtuf.coredump.cx/afl/).
The app takes a gcc-compiled binary file as an argument. 
As an example, the program tests program original_cgi.c, which takes one string input as an argument.

## How to run fuzzer:

	1. Add permissions to execute buildscript.sh with this command:
		
		chmod +x buildscript.sh

	2. Run buildscript:

		./buildscript.sh

### Under the hood, the buildscript will:

	- set up rust toolchain (rustc and cargo)
	- build fuzzer from *sources* and copy executable to the *test-app* (dir with testing C program)
	- build C program (cgi.c) with --coverage option
	- execute fuzzer over C program with option *-show* (which print out some insights while fuzzing)


