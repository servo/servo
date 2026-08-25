// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es5id: 10.4.2-1-2
description: >
    Direct call to eval has context set to local context (nested function)
---*/

var __10_4_2_1_2 = "str";
function testcase() {
            var __10_4_2_1_2 = "str1";
            function foo() {
                var __10_4_2_1_2 = "str2";
                assert(eval("\'str2\' === __10_4_2_1_2"), 'direct eval');
            }
            foo();
    }
testcase();
