// Copyright (C) 2026 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iterator.prototype.chunks
description: >
  Arguments and this value are validated in the correct order
info: |
  Iterator.prototype.chunks ( chunkSize )

  1. Let O be the this value.
  2. If O is not an Object, throw a TypeError exception.
  3. Let iterated be the Iterator Record { [[Iterator]]: O, [[NextMethod]]: undefined, [[Done]]: false }.
  4. If chunkSize is not a Number, throw a TypeError ... IteratorClose.
  5. If chunkSize is not an integral Number, throw a TypeError ... IteratorClose.
  6. If chunkSize is not in the inclusive interval from 1𝔽 to 𝔽(2^32 - 1), then
    a. Let error be ThrowCompletion(a newly created RangeError object).
    b. Return ? IteratorClose(iterated, error).
  7. Set iterated to ? GetIteratorDirect(O).

includes: [compareArray.js]
features: [iterator-chunking]
---*/
let effects = [];

// TypeError for non-object this before chunkSize is examined
assert.throws(TypeError, function () {
  Iterator.prototype.chunks.call(null, 0);
});

// RangeError for invalid chunkSize before next is accessed
assert.throws(RangeError, function () {
  Iterator.prototype.chunks.call(
    {
      get next() {
        effects.push('get next');
        return function () {
          return { done: true, value: undefined };
        };
      }
    },
    0
  );
});

assert.compareArray(effects, []);

// With valid args, next getter IS accessed (GetIteratorDirect runs)
Iterator.prototype.chunks.call(
  {
    get next() {
      effects.push('get next');
      return function () {
        return { done: true, value: undefined };
      };
    }
  },
  1
);

assert.compareArray(effects, ['get next']);
