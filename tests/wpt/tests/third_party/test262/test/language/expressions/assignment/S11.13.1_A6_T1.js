// Copyright (C) 2014 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Assignment Operator calls PutValue(lref, rval)
es5id: S11.13.1_A6_T1
description: >
    Evaluating LeftHandSideExpression lref returns Reference type; Reference
    base value is an environment record and environment record kind is
    declarative environment record. PutValue(lref, rval) uses the initially
    created Reference even if a more local binding is available.
flags: [noStrict]
---*/

function testAssignment() {
  var x = 0;
  var innerX = (function() {
    x = (eval("var x;"), 1);
    return x;
  })();

  if (innerX !== undefined) {
    throw new Test262Error('#1: innerX === undefined. Actual: ' + (innerX));
  }
  if (x !== 1) {
    throw new Test262Error('#2: x === 1. Actual: ' + (x));
  }
}
testAssignment();
