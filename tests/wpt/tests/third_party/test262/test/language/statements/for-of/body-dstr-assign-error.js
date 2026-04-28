// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-runtime-semantics-forin-div-ofheadevaluation-tdznames-expr-iterationkind
description: >
    If the left-hand side requires a DestructuringAssignment operation and that
    operation produces an error, the iterator should be closed and the error
    forwarded to the runtime.
info: |
  ...
  Else,
    If lhsKind is assignment, then
      Let status be the result of performing DestructuringAssignmentEvaluation of
      assignmentPattern using nextValue as the argument.
  ...
  If status is an abrupt completion, then
    Set the running execution context's LexicalEnvironment to oldEnv.
    If iterationKind is enumerate, then
      Return status.

features: [destructuring-assignment, for-of, Symbol.iterator]
---*/

var callCount = 0;
var iterationCount = 0;
var iterable = {};
var x = {
  set attr(_) {
    throw new Test262Error();
  }
};

iterable[Symbol.iterator] = function() {
  return {
    next: function() {
      return { done: false, value: [0] };
    },
    return: function() {
      callCount += 1;
    }
  }
};

assert.throws(Test262Error, function() {
  for ([x.attr] of iterable) {
    iterationCount += 1;
  }
});

assert.sameValue(iterationCount, 0, 'The loop body is not evaluated');
assert.sameValue(callCount, 1, 'Iterator is closed');
