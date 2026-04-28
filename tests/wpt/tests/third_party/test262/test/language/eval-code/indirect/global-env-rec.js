// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es5id: 10.4.2-1-1
description: Indirect call to eval has context set to global context
---*/

var __10_4_2_1_1_1 = "str";
function testcase() {
    var _eval = eval;
    var __10_4_2_1_1_1 = "str1";
    assert(_eval("\'str\' === __10_4_2_1_1_1"), 'indirect eval');
}
testcase();
