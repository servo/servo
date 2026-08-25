// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-delete-operator-runtime-semantics-evaluation
description: >
  ToPropertyKey not performed when deleting a super reference.
info: |
  13.5.1.2 Runtime Semantics: Evaluation

    UnaryExpression : delete UnaryExpression

    1. Let ref be ? Evaluation of UnaryExpression.
    ...
    4. If IsPropertyReference(ref) is true, then
      ...
      b. If IsSuperReference(ref) is true, throw a ReferenceError exception.
---*/

var key = {
  toString() {
    throw new Test262Error("ToPropertyKey performed");
  }
};

var obj = {
  m() {
    delete super[key];
  }
};

assert.throws(ReferenceError, () => obj.m());
