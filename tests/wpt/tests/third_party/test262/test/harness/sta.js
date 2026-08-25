// Copyright (c) 2017 Rick Waldron.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
    Including sta.js will expose three functions:

        Test262Error
        Test262Error.thrower
        $DONOTEVALUATE
---*/

assert(typeof Test262Error === "function");
assert(typeof Test262Error.prototype.toString === "function");
assert(typeof Test262Error.thrower === "function");
assert(typeof $DONOTEVALUATE === "function");
