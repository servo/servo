// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-weakmap-iterable
description: >
  Return IteratorClose(iter, status) if fail on adding value on constructing.
info: |
  WeakMap ( [ iterable ] )

  ...
  9. Repeat
    k. Let status be Call(adder, map, «k.[[value]], v.[[value]]»).
    l. If status is an abrupt completion, return IteratorClose(iter, status).
features: [Symbol.iterator]
---*/

var count = 0;
var iterable = {};
iterable[Symbol.iterator] = function() {
  return {
    next: function() {
      return {
        value: [],
        done: false
      };
    },
    return: function() {
      count += 1;
    }
  };
};
WeakMap.prototype.set = function() {
  throw new Test262Error();
};

assert.throws(Test262Error, function() {
  new WeakMap(iterable);
});

assert.sameValue(
  count, 1,
  'The iterator is closed when `WeakMap.prototype.set` throws an error.'
);
