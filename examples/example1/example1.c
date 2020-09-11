#include <stdio.h>
#include <stdlib.h>
#include <string.h>

volatile int b;
volatile int x;

void bug(void) {
    x = 123;
}

int main(int argc, char **argv, char **envp) {
    char a,b,c,v;

    printf("Input first character (a): \n");
    scanf("%c", &a);

    printf("Input second character (b): \n");
    scanf("%c", &b);

    printf("Input third character (c): \n");
    scanf("%c", &c);

    if (a == 'A') {
        if (b == 'B') {
            if(c == 'C') {
                bug();
            }
        }
    }

    return 0;
}
