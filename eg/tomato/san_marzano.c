#include <tomato.h>
#include <grow.h>

char *tomato_san_marzano() {
    if (
        grow_water() > 1
        && grow_soil() > 3
        && grow_seed() > 2
    ) {
        return "san marzano";
    }
    return "no san marzano";
}