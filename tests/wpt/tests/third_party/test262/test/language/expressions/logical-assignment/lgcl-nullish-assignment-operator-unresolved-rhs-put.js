// Copyright (c) 2020 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-assignment-operators-runtime-semantics-evaluation
description: >
    ReferenceError is thrown if the AssignmentExpression of a Logical
    Assignment operator(??=) evaluates to an unresolvable reference and the
    AssignmentExpression is evaluated.
features: [logical-assignment-operators]

---*/

var value = undefined;

assert.throws(ReferenceError, function() {
  value ??= unresolved;
});
assert.sameValue(value, undefined, "value");
