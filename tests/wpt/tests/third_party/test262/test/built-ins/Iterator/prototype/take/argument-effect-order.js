// Copyright (C) 2023 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.take
description: >
  Arguments and this value are evaluated in the correct order
info: |
  %Iterator.prototype%.take ( limit )

  1. Let O be the this value.
  2. If O is not an Object, throw a TypeError exception.
  3. Let numLimit be ? ToNumber(limit).
  4. If numLimit is NaN, throw a RangeError exception.
  5. Let integerLimit be ! ToIntegerOrInfinity(numLimit).
  6. If integerLimit < 0, throw a RangeError exception.
  7. Let iterated be ? GetIteratorDirect(O).

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
    NaN
  );
});

assert.compareArray(effects, []);
