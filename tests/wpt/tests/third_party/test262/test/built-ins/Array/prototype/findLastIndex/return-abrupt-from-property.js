// Copyright (C) 2021 Microsoft. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.findlastindex
description: >
  Returns abrupt from getting property value from `this`.
info: |
  Array.prototype.findLastIndex ( predicate[ , thisArg ] )

  ...
  4. Let k be len - 1.
  5. Repeat, while k ≥ 0,
    a. Let Pk be ! ToString(𝔽(k)).
    b. Let kValue be ? Get(O, Pk).
  ...
features: [array-find-from-last]
---*/

var o = {
  length: 1
};

Object.defineProperty(o, 0, {
  get: function() {
    throw new Test262Error();
  }
});

assert.throws(Test262Error, function() {
  [].findLastIndex.call(o, function() {});
});
