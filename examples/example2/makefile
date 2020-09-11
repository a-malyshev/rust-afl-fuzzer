clean:
	rm -f *.exe	*.gcda *.gcno *.gcov example example.c

build:
	cp ./example*.c example.c; gcc --coverage -o example ./example.c
