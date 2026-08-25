// Copyright (C) 2013 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-getiterator
description: >
    The iterator's `next` method should be accessed only once with each
    iteration as per the `GetIterator` abstract operation (7.4.1).
features: [Symbol.iterator, for-of]
---*/

var iterable = {};
var iterator = {};
var iterationCount = 0;
var loadNextCount = 0;

iterable[Symbol.iterator] = function() {
  return iterator;
};

function next() {
  if (iterationCount) return { done: true };
  return { value: 45, done: false };
}
Object.defineProperty(iterator, 'next', {
  get() { loadNextCount++; return next; },
  configurable: true
});

for (var x of iterable) {
  assert.sameValue(x, 45);

  Object.defineProperty(iterator, 'next', {
    get: function() {
      throw new Test262Error(
          'Should not access the `next` method after the iteration prologue.');
    }
  });
  iterationCount++;
}
assert.sameValue(iterationCount, 1);
assert.sameValue(loadNextCount, 1);
