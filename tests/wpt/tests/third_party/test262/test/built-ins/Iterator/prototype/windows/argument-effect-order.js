// Copyright (C) 2026 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iterator.prototype.windows
description: >
  Arguments and this value are validated in the correct order
info: |
  Iterator.prototype.windows ( windowSize [ , undersized ] )

  1. Let O be the this value.
  2. If O is not an Object, throw a TypeError exception.
  3. Let iterated be the Iterator Record { [[Iterator]]: O, [[NextMethod]]: undefined, [[Done]]: false }.
  4. If windowSize is not a Number, throw a TypeError ... IteratorClose.
  5. If windowSize is not an integral Number, throw a TypeError ... IteratorClose.
  6. If windowSize is not in the inclusive interval from 1𝔽 to 𝔽(2^32 - 1), then
    a. Let error be ThrowCompletion(a newly created RangeError object).
    b. Return ? IteratorClose(iterated, error).
  7. If undersized is undefined, set undersized to "only-full".
  8. If undersized is neither "only-full" nor "allow-partial", then
    a. Let error be ThrowCompletion(a newly created TypeError object).
    b. Return ? IteratorClose(iterated, error).
  9. Set iterated to ? GetIteratorDirect(O).

includes: [compareArray.js]
features: [iterator-chunking]
---*/
let effects = [];

// TypeError for non-object this before windowSize is examined
assert.throws(TypeError, function () {
  Iterator.prototype.windows.call(null, 0, 'bad');
});

// RangeError for invalid windowSize before undersized is examined
assert.throws(RangeError, function () {
  Iterator.prototype.windows.call(
    {
      get next() {
        effects.push('get next');
        return function () {
          return { done: true, value: undefined };
        };
      }
    },
    0,
    'bad'
  );
});

assert.compareArray(effects, []);

// TypeError for invalid undersized before next is accessed
assert.throws(TypeError, function () {
  Iterator.prototype.windows.call(
    {
      get next() {
        effects.push('get next');
        return function () {
          return { done: true, value: undefined };
        };
      }
    },
    1,
    'bad'
  );
});

assert.compareArray(effects, []);

// With all valid args, next getter IS accessed (GetIteratorDirect runs)
Iterator.prototype.windows.call(
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
