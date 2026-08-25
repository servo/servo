// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorclose
description: >
  If retrieving an iterator's `return` method generates an error while
  closing the iterator with non-throw completion, the error should be
  forwarded to the runtime.
info: |
  IteratorClose ( iteratorRecord, completion )

  [...]
  4. Let innerResult be GetMethod(iterator, "return").
  5. If innerResult.[[Type]] is normal,
    [...]
  6. If completion.[[Type]] is throw, return Completion(completion).
  7. If innerResult.[[Type]] is throw, return Completion(innerResult).

  GetMethod ( V, P )

  [...]
  2. Let func be ? GetV(V, P).
features: [Symbol.iterator]
---*/

var iterable = {};
var iterationCount = 0;

iterable[Symbol.iterator] = function() {
  return {
    next: function() {
      return { done: false, value: null };
    },
    get return() {
      throw new Test262Error();
    }
  };
};

assert.throws(Test262Error, function() {
  for (var x of iterable) {
    iterationCount += 1;
    break;
  }
});

assert.sameValue(iterationCount, 1, 'The loop body is evaluated');
