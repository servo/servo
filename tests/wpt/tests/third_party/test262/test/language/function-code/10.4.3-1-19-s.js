// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 10.4.3-1-19-s
description: >
    Strict Mode - checking 'this' (indirect eval used within strict
    mode)
flags: [onlyStrict]
---*/

var global = this;

function testcase() {
var my_eval = eval;
assert.sameValue(my_eval("this"), global, 'my_eval("this")');
}
testcase();
