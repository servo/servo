// Copyright (C) 2014 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Compound Assignment Operator calls PutValue(lref, v)
es5id: S11.13.2_A6.5_T1
description: >
    Evaluating LeftHandSideExpression lref returns Reference type; Reference
    base value is an environment record and environment record kind is
    declarative environment record. PutValue(lref, v) uses the initially
    created Reference even if a more local binding is available.
    Check operator is "x -= y".
flags: [noStrict]
---*/

function testCompoundAssignment() {
  var x = 5;
  var innerX = (function() {
    x -= (eval("var x = 2;"), 1);
    return x;
  })();

  if (innerX !== 2) {
    throw new Test262Error('#1: innerX === 2. Actual: ' + (innerX));
  }
  if (x !== 4) {
    throw new Test262Error('#2: x === 4. Actual: ' + (x));
  }
}
testCompoundAssignment();
