// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-weakmap-iterable
description: >
  Closes iterator if item second entry completes abruptly.
info: |
  WeakMap ( [ iterable ] )

  ...
  9. Repeat
    ...
    d. Let nextItem be IteratorValue(next).
    ...
    i. Let v be Get(nextItem, "1").
    j. If v is an abrupt completion, return IteratorClose(iter, v).
    ...
features: [Symbol.iterator]
---*/

var count = 0;
var item = ['foo', 'bar'];
Object.defineProperty(item, 1, {
  get: function() {
    throw new Test262Error();
  }
});
var iterable = {};
iterable[Symbol.iterator] = function() {
  return {
    next: function() {
      return {
        value: item,
        done: false
      };
    },
    return: function() {
      count++;
    }
  };
};

assert.throws(Test262Error, function() {
  new WeakMap(iterable);
});

assert.sameValue(count, 1, 'The get error closed the iterator');
