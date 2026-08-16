// Copyright (C) 2026 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iterator.prototype.includes
description: >
  Arguments and this value are validated in the correct order
info: |
  Iterator.prototype.includes ( searchElement [ , skippedElements ] )

  1. Let O be the this value.
  2. If O is not an Object, throw a TypeError exception.
  3. Let iterated be the Iterator Record { [[Iterator]]: O, [[NextMethod]]: undefined, [[Done]]: false }.
  4. If skippedElements is undefined, let toSkip be 0.
  5. Else if skippedElements is not one of +Infinity, -Infinity, or an integral Number,
    a. Let error be ThrowCompletion(a newly created TypeError object).
    b. Return ? IteratorClose(iterated, error).
  6. Else, let toSkip be skippedElements.
  7. If toSkip < -0F, then
    a. Let error be ThrowCompletion(a newly created RangeError object).
    b. Return ? IteratorClose(iterated, error).
  8. If toSkip is finite and toSkip > F(2**53 - 1), then
    a. Let error be ThrowCompletion(a newly created RangeError object).
    b. Return ? IteratorClose(iterated, error).
  9. Let skipped be +0F.
  10. Set iterated to ? GetIteratorDirect(O).

includes: [compareArray.js]
features: [iterator-includes]
---*/

assert.throws(TypeError, function() {
  Iterator.prototype.includes.call(null, 0, NaN);
});

let effects = [];

assert.throws(TypeError, function() {
  Iterator.prototype.includes.call(
    {
      get next() {
        effects.push('get next');
        return function() {
          return { done: true, value: undefined };
        };
      },
      return() {
        effects.push('return');
        return {};
      },
    },
    0,
    NaN
  );
});

assert.compareArray(effects, ['return']);

effects = [];

assert.throws(RangeError, function() {
  Iterator.prototype.includes.call(
    {
      get next() {
        effects.push('get next');
        return function() {
          return { done: true, value: undefined };
        };
      },
      return() {
        effects.push('return');
        return {};
      },
    },
    0,
    Number.MAX_SAFE_INTEGER + 1
  );
});

assert.compareArray(effects, ['return']);

effects = [];

Iterator.prototype.includes.call(
  {
    get next() {
      effects.push('get next');
      return function() {
        return { done: true, value: undefined };
      };
    },
  },
  0,
  0
);

assert.compareArray(effects, ['get next']);
