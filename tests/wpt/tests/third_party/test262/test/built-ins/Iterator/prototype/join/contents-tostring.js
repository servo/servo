// Copyright (C) 2025 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-iterator.prototype.join
description: Iterator.prototype.join coerces non-nullish iterator contents to string.
features: [Iterator.prototype.join]
---*/

var called = false;
var coercible = {
  toString: function () {
    if (called) {
      throw new Test262Error('toString should be called exactly once');
    }
    called = true;
    return 'value';
  },
};

assert.sameValue([coercible, 0, true].values().join(), 'value,0,true');
assert(called);
