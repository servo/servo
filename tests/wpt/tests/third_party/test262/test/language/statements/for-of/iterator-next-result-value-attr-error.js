// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 13.6.4.13 S5.g
description: >
    If `nextValue` is an abrupt completion as per IteratorValue (ES6 7.4.4),
    return the completion.
info: |
  [...]
  5. Repeat
     a. Let nextResult be ? IteratorStep(iterator).
     b. If nextResult is false, return NormalCompletion(V).
     c. Let nextValue be ? IteratorValue(nextResult).
features: [Symbol.iterator]
---*/

var iterable = {};
var iterationCount = 0;
var returnCount = 0;

iterable[Symbol.iterator] = function() {
  return {
    next: function() {
      return {
        done: false,
        get value() {
          throw new Test262Error();
        }
      };
    },
    return: function() {
      returnCount += 1;
      return {};
    }
  };
};

assert.throws(Test262Error, function() {
  for (var x of iterable) {
    iterationCount += 1;
  }
});

assert.sameValue(iterationCount, 0, 'The loop body is not evaluated');
assert.sameValue(returnCount, 0, 'Iterator is not closed.');
