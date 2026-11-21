// Copyright (C) 2026 Saúl Ibarra Corretgé. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-iterator.concat
description: >
  Throws a TypeError when the underlying iterator's return method returns a non-object
features: [iterator-sequencing]
---*/

let returnCalls = 0;

const testIterator = {
  next() {
    return {
      done: false,
      value: 1,
    };
  },
  return() {
    ++returnCalls;
    return 1;
  },
};

const iterator = Iterator.concat({
  [Symbol.iterator]() {
    return testIterator;
  }
});

iterator.next();

assert.throws(TypeError, function() {
  iterator.return();
});

assert.sameValue(returnCalls, 1);

// The iterator helper is completed even though closing the underlying iterator
// failed.
const resultAfterThrow = iterator.next();

assert.sameValue(resultAfterThrow.done, true);
assert.sameValue(resultAfterThrow.value, undefined);

const resultAfterReturn = iterator.return();

assert.sameValue(resultAfterReturn.done, true);
assert.sameValue(resultAfterReturn.value, undefined);
assert.sameValue(returnCalls, 1, 'return is not called again on the underlying iterator');
