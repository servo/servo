// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: use-strict-directive
es5id: 10.1.1-24-s
description: >
    Strict Mode - Function code of a FunctionExpression contains Use
    Strict Directive which appears at the end of the block
flags: [noStrict]
---*/

        (function () {
            test262unresolvable = null;
            "use strict";

            assert.sameValue(test262unresolvable, null);
        }) ();
