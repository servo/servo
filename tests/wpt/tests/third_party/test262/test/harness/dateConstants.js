// Copyright (c) 2017 Rick Waldron.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
    Including dateConstants.js will expose:

        var date_1899_end = -2208988800001;
        var date_1900_start = -2208988800000;
        var date_1969_end = -1;
        var date_1970_start = 0;
        var date_1999_end = 946684799999;
        var date_2000_start = 946684800000;
        var date_2099_end = 4102444799999;
        var date_2100_start = 4102444800000;

includes: [dateConstants.js]
---*/
assert.sameValue(date_1899_end, -2208988800001);
assert.sameValue(date_1900_start, -2208988800000);
assert.sameValue(date_1969_end, -1);
assert.sameValue(date_1970_start, 0);
assert.sameValue(date_1999_end, 946684799999);
assert.sameValue(date_2000_start, 946684800000);
assert.sameValue(date_2099_end, 4102444799999);
assert.sameValue(date_2100_start, 4102444800000);
