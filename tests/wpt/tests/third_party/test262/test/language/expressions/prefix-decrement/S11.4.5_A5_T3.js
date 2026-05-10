// Copyright (C) 2014 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Operator --x calls PutValue(lhs, newValue)
es5id: S11.4.5_A5_T3
description: >
    Evaluating LeftHandSideExpression lhs returns Reference type; Reference
    base value is an environment record and environment record kind is
    object environment record. PutValue(lhs, newValue) uses the initially
    created Reference even if the environment binding is no longer present.
    Binding in surrounding object environment record is not changed.
flags: [noStrict]
---*/

var outerScope = {
  x: 0
};
var innerScope = {
  get x() {
    delete this.x;
    return 2;
  }
};

with (outerScope) {
  with (innerScope) {
    --x;
  }
}

if (innerScope.x !== 1) {
  throw new Test262Error('#1: innerScope.x === 1. Actual: ' + (innerScope.x));
}
if (outerScope.x !== 0) {
  throw new Test262Error('#2: outerScope.x === 0. Actual: ' + (outerScope.x));
}
