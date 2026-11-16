// Copyright (C) 2026 Saúl Ibarra Corretgé. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.flatMap
description: >
  Only the underlying iterator is closed when stepping the inner iterator throws
features: [iterator-helpers, class]
---*/

// An abrupt completion from stepping the inner iterator goes through
// IfAbruptCloseIterator, which closes the underlying iterator, but not the
// inner iterator itself.
let outerReturnCalls = 0;
let innerReturnCalls = 0;

class TestIterator extends Iterator {
  next() {
    return {
      done: false,
      value: 1,
    };
  }
  return() {
    ++outerReturnCalls;
    return {};
  }
}

const innerIterator = {
  [Symbol.iterator]() {
    return this;
  },
  next() {
    throw new Test262Error();
  },
  return() {
    ++innerReturnCalls;
    return {};
  },
};

const iterator = new TestIterator().flatMap(() => innerIterator);

assert.throws(Test262Error, function () {
  iterator.next();
});

assert.sameValue(outerReturnCalls, 1, 'the underlying iterator is closed');
assert.sameValue(innerReturnCalls, 0, 'the inner iterator is not closed');
