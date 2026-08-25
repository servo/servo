// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-runtime-semantics-forin-div-ofbodyevaluation-lhs-stmt-iterator-lhskind-labelset
description: >
  The left-hand side may take the form of a DestructuringAssignment.
info: |
  ...
  Else,
    If lhsKind is assignment, then
      Let status be the result of performing DestructuringAssignmentEvaluation of
      assignmentPattern using nextValue as the argument.

features: [destructuring-assignment, for-of]
---*/

var iterationCount = 0;
var x;

for ([x] of [[0]]) {
  assert.sameValue(x, 0);
  iterationCount += 1;
}

assert.sameValue(iterationCount, 1);
