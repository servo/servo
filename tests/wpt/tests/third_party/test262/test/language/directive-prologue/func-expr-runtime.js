// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: use-strict-directive
es5id: 10.1.1-22-s
description: >
    Strict Mode - Function code of a FunctionExpression contains Use
    Strict Directive which appears at the start of the block
flags: [noStrict]
---*/

assert.throws(ReferenceError, function () {
            "use strict";

            test262unresolvable = null;
});
