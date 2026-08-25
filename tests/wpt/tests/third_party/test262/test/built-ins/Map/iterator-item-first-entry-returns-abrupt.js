// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-map-iterable
description: >
  Closes iterator if item first entry completes abruptly.
info: |
  Map ( [ iterable ] )

  ...
  9. Repeat
    ...
    d. Let nextItem be IteratorValue(next).
    ...
    g. Let k be Get(nextItem, "0").
    h. If k is an abrupt completion, return IteratorClose(iter, k).
    ...
features: [Symbol.iterator]
---*/

var count = 0;
var item = ['foo', 'bar'];
Object.defineProperty(item, 0, {
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
  new Map(iterable);
});

assert.sameValue(count, 1, 'The get error closed the iterator');
