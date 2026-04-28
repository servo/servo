// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 10.5-7-b-4-s
description: >
    Deleting property of the arguments object successful under strict mode
---*/

function _10_5_7_b_4_fun() {
    var _10_5_7_b_4_1 = arguments[0] === 30 && arguments[1] === 12;
    delete arguments[1];
    var _10_5_7_b_4_2 = arguments[0] === 30 && typeof arguments[1] === "undefined";
    return _10_5_7_b_4_1 && _10_5_7_b_4_2;
};

assert(_10_5_7_b_4_fun(30, 12), '_10_5_7_b_4_fun(30, 12) !== true');
