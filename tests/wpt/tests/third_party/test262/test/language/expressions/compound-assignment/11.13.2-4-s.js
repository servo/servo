// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 11.13.2-4-s
description: >
    Strict Mode - ReferenceError is thrown if the
    LeftHandSideExpression of a Compound Assignment operator(+=)
    evaluates to an unresolvable reference
---*/


assert.throws(ReferenceError, function() {
            eval("_11_13_2_4 += 1;");
});
