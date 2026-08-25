// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 21.2.5.9
description: The `lastIndex` value is restored following match execution
info: |
    [...]
    8. If SameValue(currentLastIndex, previousLastIndex) is false, then
       a. Perform ? Set(rx, "lastIndex", previousLastIndex, true).
    [...]
features: [Symbol.search]
---*/

var latestValue = 86;
var callCount = 0;
var fakeRe = {
  get lastIndex() {
    return latestValue;
  },
  set lastIndex(_) {
    latestValue = _;
  },
  exec: function() {
    callCount++;
    latestValue = null;
    return null;
  }
};

RegExp.prototype[Symbol.search].call(fakeRe);

assert.sameValue(callCount, 1);
assert.sameValue(latestValue, 86);
