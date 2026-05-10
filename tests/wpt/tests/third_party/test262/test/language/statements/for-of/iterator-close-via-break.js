// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 13.6.4.13
description: >
    Iterators should be closed via their `return` method when iteration is
    interrupted via a `break` statement.
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

for (var x of iterable) {
  assert.sameValue(
    startedCount, 1, 'Value is retrieved'
  );
  assert.sameValue(
    returnCount, 0, 'Iterator is not closed'
  );
  iterationCount += 1;
  break;
}

assert.sameValue(
  startedCount, 1, 'Iterator does not restart following interruption'
);
assert.sameValue(iterationCount, 1, 'A single iteration occurs');
assert.sameValue(
  returnCount, 1, 'Iterator is closed after `break` statement'
);
