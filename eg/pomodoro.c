#include <stdio.h>
#include <string.h>
#include <grow.h>
#include <tomato.h>

int main(int argc, char **argv) {
    if (argc != 2) {
        printf("pomodoro takes one arg\n");
        return 1;
    }

    if (!strcmp(argv[1], "beefmaster")) {
        printf("%s\n", tomato_beefmaster());
    } else if (!strcmp(argv[1], "san marzano")) {
        printf("%s\n", tomato_san_marzano());
    } else {
        printf("unknown tomato: %s\n", argv[1]);
        return 1;
    }

    return 0;
}
