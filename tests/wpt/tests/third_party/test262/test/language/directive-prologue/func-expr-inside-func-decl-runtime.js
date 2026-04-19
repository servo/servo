// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: use-strict-directive
es5id: 10.1.1-16-s
description: >
    Strict Mode - Function code that is part of a FunctionExpression
    is strict function code if FunctionExpression is contained in use
    strict
flags: [noStrict]
---*/

function testcase() {
        "use strict";

        assert.throws(ReferenceError, function() {
            test262unresolvable = null;
        });
    }
testcase();
