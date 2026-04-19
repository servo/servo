// Copyright (C) 2023 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iterator.from
description: >
  Iterator.from does not respect the iterability of any primitive except Strings
info: |
  Iterator.from ( O )

  1. If O is a String, set O to ! ToObject(O).
  2. Let iteratorRecord be ? GetIteratorFlattenable(O).

includes: [compareArray.js]
features: [iterator-helpers]
flags: []
---*/

function* g() {
  yield 0;
}

Number.prototype[Symbol.iterator] = function* () {
  let i = 0;
  let target = this >>> 0;
  while (i < target) {
    yield i;
    ++i;
  }
};

assert.compareArray(Array.from(5), [0, 1, 2, 3, 4]);

assert.throws(TypeError, function () {
  Iterator.from(5);
});

assert.compareArray(Array.from(Iterator.from(new Number(5))), [0, 1, 2, 3, 4]);

assert.compareArray(Array.from(Iterator.from('string')), ['s', 't', 'r', 'i', 'n', 'g']);

const originalStringIterator = String.prototype[Symbol.iterator];
let observedType;
Object.defineProperty(String.prototype, Symbol.iterator, {
  get() {
    'use strict';
    observedType = typeof this;
    return originalStringIterator;
  }
});
Iterator.from('');
assert.sameValue(observedType, 'string');
Iterator.from(new String(''));
assert.sameValue(observedType, 'object');
