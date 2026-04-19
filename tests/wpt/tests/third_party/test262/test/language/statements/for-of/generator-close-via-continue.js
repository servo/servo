// Copyright (C) 2017 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-runtime-semantics-forin-div-ofbodyevaluation-lhs-stmt-iterator-lhskind-labelset
description: >
  Generators should be closed via their `return` method when iteration is
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

features: [generators]
---*/

var startedCount = 0;
var finallyCount = 0;
var iterationCount = 0;
function* values() {
  startedCount += 1;
  try {
    yield;
    throw new Test262Error('This code is unreachable (within `try` block)');
  } finally {
    finallyCount += 1;
  }
  throw new Test262Error('This code is unreachable (following `try` statement)');
}
var iterable = values();

assert.sameValue(
  startedCount, 0, 'Generator is initialized in suspended state'
);

L: do {
  for (var x of iterable) {
    assert.sameValue(
      startedCount, 1, 'Generator executes prior to first iteration'
    );
    assert.sameValue(
      finallyCount, 0, 'Generator is paused during first iteration'
    );
    iterationCount += 1;
    continue L;
  }
} while (false);

assert.sameValue(
  startedCount, 1, 'Generator does not restart following interruption'
);
assert.sameValue(iterationCount, 1, 'A single iteration occurs');
assert.sameValue(
  finallyCount, 1, 'Generator is closed after `continue` statement'
);
