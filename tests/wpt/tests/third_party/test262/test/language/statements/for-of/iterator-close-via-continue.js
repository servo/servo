// Copyright (C) 2017 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-runtime-semantics-forin-div-ofbodyevaluation-lhs-stmt-iterator-lhskind-labelset
description: >
  Iterators should be closed via their `return` method when iteration is
  interrupted via a `continue` statement.
info: |
  13.7.5.13 Runtime Semantics: ForIn/OfBodyEvaluation ( lhs, stmt, iteratorRecord, iterationKind, lhsKind, labelSet )
    ...
    5. Repeat,
      ...
      i. Let result be the result of evaluating stmt.
      ...
      k. If LoopContinues(result, labelSet) is false, then
        i. If iterationKind is enumerate, then
          ...
        ii. Else,
          1. Assert: iterationKind is iterate.
          2. Return ? IteratorClose(iteratorRecord, UpdateEmpty(result, V)).
      ...

features: [Symbol.iterator]
---*/

var startedCount = 0;
var returnCount = 0;
var iterationCount = 0;
var iterable = {};

iterable[Symbol.iterator] = function() {
  return {
    next: function() {
      startedCount += 1;
      return { done: false, value: null };
    },
    return: function() {
      returnCount += 1;
      return {};
    }
  };
};

L: do {
  for (var x of iterable) {
    assert.sameValue(
      startedCount, 1, 'Value is retrieved'
    );
    assert.sameValue(
      returnCount, 0, 'Iterator is not closed'
    );
    iterationCount += 1;
    continue L;
  }
} while (false);

assert.sameValue(
  startedCount, 1, 'Iterator does not restart following interruption'
);
assert.sameValue(iterationCount, 1, 'A single iteration occurs');
assert.sameValue(
  returnCount, 1, 'Iterator is closed after `continue` statement'
);
