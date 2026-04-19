// Copyright (c) 2020 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-assignment-operators-runtime-semantics-evaluation
description: >
    ReferenceError is not thrown if the AssignmentExpression of a Logical
    Assignment operator(??=) evaluates to an unresolvable reference and the
    AssignmentExpression is not evaluated.
features: [logical-assignment-operators]

---*/

var value = 0;

assert.sameValue(value ??= unresolved, 0, "value");
