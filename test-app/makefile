clean:
	rm -f	*.gcda *.gcno *.gcov cgi

build:
	cp ./original_cgi.c cgi.c; gcc --coverage -o cgi ./cgi.c
	#./AFL/afl-gcc -o cgi ./cgi.c 
