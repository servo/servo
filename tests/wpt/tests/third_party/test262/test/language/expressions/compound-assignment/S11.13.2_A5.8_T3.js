// Copyright (C) 2014 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Compound Assignment Operator calls PutValue(lref, v)
es5id: S11.13.2_A5.8_T3
description: >
    Evaluating LeftHandSideExpression lref returns Reference type; Reference
    base value is an environment record and environment record kind is
    object environment record. PutValue(lref, v) uses the initially
    created Reference even if the environment binding is no longer present.
    Binding in surrounding object environment record is not changed.
    Check operator is "x >>>= y".
flags: [noStrict]
---*/

var outerScope = {
  x: 0
};
var innerScope = {
  get x() {
    delete this.x;
    return 16;
  }
};

with (outerScope) {
  with (innerScope) {
    x >>>= 3;
  }
}

if (innerScope.x !== 2) {
  throw new Test262Error('#1: innerScope.x === 2. Actual: ' + (innerScope.x));
}
if (outerScope.x !== 0) {
  throw new Test262Error('#2: outerScope.x === 0. Actual: ' + (outerScope.x));
}
