// Copyright (C) 2014 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Compound Assignment Operator calls PutValue(lref, v)
es5id: S11.13.2_A5.11_T2
description: >
    Evaluating LeftHandSideExpression lref returns Reference type; Reference
    base value is an environment record and environment record kind is
    object environment record. PutValue(lref, v) uses the initially
    created Reference even if the environment binding is no longer present.
    Binding in surrounding global environment record is not changed.
    Check operator is "x |= y".
flags: [noStrict]
---*/

var x = 0;
var scope = {
  get x() {
    delete this.x;
    return 2;
  }
};

with (scope) {
  x |= 4;
}

if (scope.x !== 6) {
  throw new Test262Error('#1: scope.x === 6. Actual: ' + (scope.x));
}
if (x !== 0) {
  throw new Test262Error('#2: x === 0. Actual: ' + (x));
}
