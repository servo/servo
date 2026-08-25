// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 21.2.5.9
description: >
    Behavior when error thrown while restoring `lastIndex` property following
    match execution
info: |
    [...]
    8. If SameValue(currentLastIndex, previousLastIndex) is false, then
        a. Perform ? Set(rx, "lastIndex", previousLastIndex, true).
features: [Symbol.search]
---*/

var callCount;
var poisonedLastIndex = {
  get lastIndex() { return this.lastIndex_; },
  set lastIndex(_) {
    if (callCount === 1) {
      throw new Test262Error();
    }
    this.lastIndex_ = _;
  },
  exec: function() {
    callCount += 1;
    return null;
  }
};
var nonWritableLastIndex = {
  exec: function() {
    Object.defineProperty(
      nonWritableLastIndex, 'lastIndex', { writable: false }
    );
    callCount += 1;
    return null;
  }
};

callCount = 0;
assert.throws(Test262Error, function() {
  RegExp.prototype[Symbol.search].call(poisonedLastIndex);
});
assert.sameValue(callCount, 1, 'Match executed ("poisoned" lastIndex)');

callCount = 0;
assert.throws(TypeError, function() {
  RegExp.prototype[Symbol.search].call(nonWritableLastIndex);
});
assert.sameValue(callCount, 1, 'Match executed (non-writable lastIndex)');
