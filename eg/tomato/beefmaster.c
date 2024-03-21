#include <tomato.h>
#include <grow.h>

char *tomato_beefmaster() {
    if (
        grow_water() > 2
        && grow_soil() > 2
        && grow_seed() > 2
    ) {
        return "beefmaster";
    }
    return "no beefmaster";
}