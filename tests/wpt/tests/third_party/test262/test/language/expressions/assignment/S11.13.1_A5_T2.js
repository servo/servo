// Copyright (C) 2014 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Assignment Operator calls PutValue(lref, rval)
es5id: S11.13.1_A5_T2
description: >
    Evaluating LeftHandSideExpression lref returns Reference type; Reference
    base value is an environment record and environment record kind is
    object environment record. PutValue(lref, rval) uses the initially
    created Reference even if the environment binding is no longer present.
    Binding in surrounding global environment record is not changed.
flags: [noStrict]
---*/

var x = 0;
var scope = {x: 1};

with (scope) {
  x = (delete scope.x, 2);
}

if (scope.x !== 2) {
  throw new Test262Error('#1: scope.x === 2. Actual: ' + (scope.x));
}
if (x !== 0) {
  throw new Test262Error('#2: x === 0. Actual: ' + (x));
}
