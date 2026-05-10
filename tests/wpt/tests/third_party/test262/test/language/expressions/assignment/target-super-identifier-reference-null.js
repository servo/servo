// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-assignment-operators
description: Assignment Operator evaluates the value prior validating a SuperProperty's reference (null)
info: |
  # 13.15.2 Runtime Semantics: Evaluation
  AssignmentExpression : LeftHandSideExpression = AssignmentExpression

  1. If LeftHandSideExpression is neither an ObjectLiteral nor an ArrayLiteral,
     then
     a. Let lref be the result of evaluating LeftHandSideExpression.
     [...]
     e. Perform ? PutValue(lref, rval).

  # 6.2.4.5 PutValue ( V, W )

  [...]
  5. If IsPropertyReference(V) is true, then
     a. Let baseObj be ? ToObject(V.[[Base]]).
---*/

var count = 0;
class C {
  static m() {
    super.x = count += 1;
  }
}

Object.setPrototypeOf(C, null);

assert.throws(TypeError, function() {
  C.m();
});

assert.sameValue(count, 1);
