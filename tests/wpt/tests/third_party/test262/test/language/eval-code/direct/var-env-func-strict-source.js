// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 10.4.2.1-4-s
description: >
    Strict Mode - Strict mode eval code cannot instantiate functions
    in the variable environment of the caller to eval.
---*/

function testcase() {
        eval("'use strict'; function _10_4_2_1_4_fun(){}");
        assert.sameValue(typeof _10_4_2_1_4_fun, "undefined");
    }
testcase();
