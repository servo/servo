// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 13.6.4.13
description: >
    Generators should be closed via their `return` method when iteration is
    interrupted via a `return` statement.
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

(function() {
  for (var x of iterable) {
    assert.sameValue(
      startedCount, 1, 'Generator executes prior to first iteration'
    );
    assert.sameValue(
      finallyCount, 0, 'Generator is paused during first iteration'
    );
    iterationCount += 1;
    return;
  }
}());

assert.sameValue(
  startedCount, 1, 'Generator does not restart following interruption'
);
assert.sameValue(iterationCount, 1, 'A single iteration occurs');
assert.sameValue(
  finallyCount, 1, 'Generator is closed after `return` statement'
);
