#include <stdio.h>
#include <stdlib.h>
#include <string.h>

volatile int b;
void bug(void) {
		b = 5;
}

int main(int argc, char **argv, char **envp) {
    char a,b,c,v;

    printf("first char (a): \n");
    scanf("%c", &a);

    printf("second char (b): \n");
    scanf("%c", &b);

    printf("third char (c): \n");
    scanf("%c", &c);

    if (a < 'A' || a > 'Z') return -1;
    if (b < 'A' || b > 'Z') return -1;
    if (c < 'A' || c > 'Z') return -1;

    if (a != 'D') {
        return -1;
    }

    if (b - a != 3) {
        return -1;
    }

    if (c - b != 3) {
        return -1;
    }

    bug();
    return 0;
}
