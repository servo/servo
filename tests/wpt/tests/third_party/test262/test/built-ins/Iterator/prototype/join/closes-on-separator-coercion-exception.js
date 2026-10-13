// Copyright (C) 2025 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-iterator.prototype.join
description: Iterator.prototype.join closes its receiver if coercing the separator throws.
features: [Iterator.prototype.join]
---*/

var throwy = {
  toString: function () {
    throw new Test262Error();
  },
};

var gotNext = false;
var calledReturn = false;
var it = {
  get next() {
    // we use a variable instead of simply throwing because throwing is expected in this test
    gotNext = true;
  },
  return: function () {
    calledReturn = true;
  },
};

assert.throws(Test262Error, function () {
  Iterator.prototype.join.call(it, throwy);
});

assert.sameValue(gotNext, false);
assert(calledReturn);
