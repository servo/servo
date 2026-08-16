// Copyright (C) 2023 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iterator.prototype.take
description: >
  Arguments and this value are evaluated in the correct order
info: |
  Iterator.prototype.take ( limit )

  1. Let obj be the this value.
  2. If obj is not an Object, throw a TypeError exception.
  3. Let iterated be the Iterator Record { [[Iterator]]: obj, [[NextMethod]]: undefined, [[Done]]: false }.
  4. Let numLimit be Completion(ToNumber(limit)).
  5. IfAbruptCloseIterator(numLimit, iterated).
  6. If numLimit is NaN, then
     a. Let error be ThrowCompletion(a newly created RangeError object).
     b. Return ? IteratorClose(iterated, error).
  7. If numLimit is finite and numLimit > 𝔽(2**53 - 1), then
     a. Let error be ThrowCompletion(a newly created RangeError object).
     b. Return ? IteratorClose(iterated, error).
  8. Let integerLimit be ! ToIntegerOrInfinity(numLimit).
  9. If integerLimit < 0, then
     a. Let error be ThrowCompletion(a newly created RangeError object).
     b. Return ? IteratorClose(iterated, error).
  10. Set iterated to ? GetIteratorDirect(obj).

includes: [compareArray.js]
features: [iterator-helpers]
flags: []
---*/
let effects = [];

Iterator.prototype.take.call(
  {
    get next() {
      effects.push('get next');
      return function () {
        return { done: true, value: undefined };
      };
    },
  },
  {
    valueOf() {
      effects.push('ToNumber limit');
      return 0;
    },
  }
);

assert.compareArray(effects, ['ToNumber limit', 'get next']);

effects = [];

assert.throws(TypeError, function () {
  Iterator.prototype.take.call(null, {
    valueOf() {
      effects.push('ToNumber limit');
      return 0;
    },
  });
});

assert.compareArray(effects, []);

effects = [];

assert.throws(RangeError, function () {
  Iterator.prototype.take.call(
    {
      get next() {
        effects.push('get next');
        return function () {
          return { done: true, value: undefined };
        };
      },
    },
    {
      valueOf() {
        effects.push('ToNumber limit');
        return Number.MAX_SAFE_INTEGER + 1;
      },
    }
  );
});

assert.compareArray(effects, ['ToNumber limit']);

effects = [];

assert.throws(RangeError, function () {
  Iterator.prototype.take.call(
    {
      get next() {
        effects.push('get next');
        return function () {
          return { done: true, value: undefined };
        };
      },
    },
    NaN
  );
});

assert.compareArray(effects, []);
