// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 10.5-7-b-3-s
description: >
    Adding property to the arguments object successful under strict mode
---*/

function _10_5_7_b_3_fun() {
    arguments[1] = 12;
    return arguments[0] === 30 && arguments[1] === 12;
};

assert(_10_5_7_b_3_fun(30), '_10_5_7_b_3_fun(30) !== true');
