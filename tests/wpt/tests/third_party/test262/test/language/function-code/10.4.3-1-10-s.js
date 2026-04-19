// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 10.4.3-1-10-s
description: >
    Strict Mode - checking 'this' (FunctionExpression includes strict
    directive prologue)
flags: [noStrict]
---*/

var f = function () {
    "use strict";
    return typeof this;
}

assert.sameValue(f(), "undefined", 'f()');
