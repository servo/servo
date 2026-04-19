// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 10.4.3-1-23-s
description: >
    Strict Mode - checking 'this' (New'ed object from
    FunctionExpression defined within strict mode)
flags: [onlyStrict]
---*/

var f = function () {
    return this;
}

assert.notSameValue((new f()), this, '(new f())');
assert.notSameValue(typeof (new f()), "undefined", 'typeof (new f())');
