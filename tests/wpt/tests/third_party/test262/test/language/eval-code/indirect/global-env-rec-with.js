// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es5id: 10.4.2-1-4
description: >
    Indirect call to eval has context set to global context (with
    block)
flags: [noStrict]
---*/

var __10_4_2_1_4 = "str";
function testcase() {
            var o = new Object();
            o.__10_4_2_1_4 = "str2";
            var _eval = eval;
            var __10_4_2_1_4 = "str1";
            with (o) {
                assert(_eval("\'str\' === __10_4_2_1_4"), 'indirect eval');
            }
    }
testcase();
