/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#include "servo/servo_capi.h"
#include <assert.h>
#include <stdlib.h>

int run_c_api_tests(void) {
    /* Test servo_builder_create and servo_builder_free */
    ServoBuilder *builder = servo_builder_create();
    assert(builder != NULL);
    servo_builder_free(builder);

    return 0;
}
