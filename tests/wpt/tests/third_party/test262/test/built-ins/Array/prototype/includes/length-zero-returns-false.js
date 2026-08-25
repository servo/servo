// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.includes
description: Returns false if length is 0
info: |
  22.1.3.11 Array.prototype.includes ( searchElement [ , fromIndex ] )

  ...
  2. Let len be ? ToLength(? Get(O, "length")).
  3. If len is 0, return false.
  ...
features: [Array.prototype.includes]
---*/

var calls = 0;
var fromIndex = {
  valueOf: function() {
    calls++;
  }
};

var sample = [];
assert.sameValue(sample.includes(0), false, "returns false");
assert.sameValue(sample.includes(), false, "returns false - no arg");
assert.sameValue(sample.includes(0, fromIndex), false, "using fromIndex");
assert.sameValue(calls, 0, "length is checked before ToInteger(fromIndex)");
