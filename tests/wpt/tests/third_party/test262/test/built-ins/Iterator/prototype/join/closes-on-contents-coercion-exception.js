// Copyright (C) 2025 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-iterator.prototype.join
description: Iterator.prototype.join closes its receiver if coercing a value from the iterator throws.
features: [Iterator.prototype.join]
---*/

var throwy = {
  toString: function () {
    throw new Test262Error();
  },
};

var calledNextCount = 0;
var calledReturn = false;
var it = {
  next: function () {
    ++calledNextCount;
    if (calledNextCount > 1) {
      return { done: true, value: undefined };
    }
    return { done: false, value: throwy };
  },
  return: function () {
    calledReturn = true;
  },
};

assert.throws(Test262Error, function () {
  Iterator.prototype.join.call(it);
});

assert.sameValue(calledNextCount, 1);
assert(calledReturn);
