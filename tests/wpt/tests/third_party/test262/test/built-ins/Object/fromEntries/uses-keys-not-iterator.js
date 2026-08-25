// Copyright (C) 2018 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Reads properties rather than iterating.
esid: sec-object.fromentries
features: [Symbol.iterator, Object.fromEntries]
---*/

var iterable = {
  [Symbol.iterator]: function() {
    var count = 0;
    return {
      next: function() {
        if (count === 0) {
          ++count;
          return {
            done: false,
            value: {
              '0': 'first key',
              '1': 'first value',
              get [Symbol.iterator]() {
                throw new Test262Error('Object.fromEntries should not access Symbol.iterator on entry objects');
              },
            },
          };
        } else if (count === 1) {
          ++count;
          Array.prototype[Symbol.iterator] = function() {
            throw new Test262Error('Object.fromEntries should not access Symbol.iterator on entry arrays');
          };
          return {
            done: false,
            value: ['second key', 'second value'],
          };
        } else {
          return {
            done: true,
          };
        }
      },
    };
  },
};

var result = Object.fromEntries(iterable);
assert.sameValue(result['first key'], 'first value');
assert.sameValue(result['second key'], 'second value');
