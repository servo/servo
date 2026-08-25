// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-weakset-iterable
description: >
  Return IteratorClose(iter, status) if fail on adding value on constructing.
info: |
  WeakSet ( [ iterable ] )

  ...
  9. Repeat
    f. Let status be Call(adder, set, «nextValue»).
    g. If status is an abrupt completion, return IteratorClose(iter, status).
features: [Symbol.iterator]
---*/

var count = 0;
var iterable = {};
iterable[Symbol.iterator] = function() {
  return {
    next: function() {
      return {
        value: null,
        done: false
      };
    },
    return: function() {
      count += 1;
    }
  };
};
WeakSet.prototype.add = function() {
  throw new Test262Error();
};

assert.throws(Test262Error, function() {
  new WeakSet(iterable);
});

assert.sameValue(
  count, 1,
  'The iterator is closed when `WeakSet.prototype.add` throws an error.'
);
