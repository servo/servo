// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 10.6-12-1
description: Accessing callee property of Arguments object is allowed
flags: [noStrict]
---*/

function testcase() {
    arguments.callee;
}
testcase();
