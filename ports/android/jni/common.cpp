#include "common.h"
#include <stdlib.h>

void free_ptrarray(void **pa)
{
    void **rover = pa;

    while (*rover != NULL)
        free(*rover++);

    free(pa);
}


