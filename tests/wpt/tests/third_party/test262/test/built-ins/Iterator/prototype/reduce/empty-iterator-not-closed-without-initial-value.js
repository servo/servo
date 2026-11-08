// Copyright (C) 2026 Saúl Ibarra Corretgé. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.reduce
description: >
  Underlying iterator is not closed when reduce throws for an empty iterator
features: [iterator-helpers, class]
---*/

// The iterator has been exhausted by the time reduce throws for the missing
// initial value, so it is not closed.
let returnCalls = 0;

class TestIterator extends Iterator {
  next() {
    return {
      done: true,
      value: undefined,
    };
  }
  return() {
    ++returnCalls;
    return {};
  }
}

const iterator = new TestIterator();

assert.throws(TypeError, function () {
  iterator.reduce(() => {});
});

assert.sameValue(returnCalls, 0, 'return is not called on the underlying iterator');
