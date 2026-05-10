// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 10.6-6-4
description: >
    'length' property of arguments object for 0 argument function call
    is 0 even with formal parameters
---*/

function testcase(a,b,c) {
    assert.sameValue(arguments.length, 0);
}
testcase();
