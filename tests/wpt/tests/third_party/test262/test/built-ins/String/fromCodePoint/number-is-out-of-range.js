// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 21.1.2.2
description: >
  Throw a RangeError if an argument is < 0 or > 0x10FFFF.
info: |
  String.fromCodePoint ( ...codePoints )

  1. Let result be the empty String.
  2. For each element next of codePoints, do
    a. Let nextCP be ? ToNumber(next).
    b. If nextCP is not an integral Number, throw a RangeError exception.
    c. If ℝ(nextCP) < 0 or ℝ(nextCP) > 0x10FFFF, throw a RangeError exception.
  ...
features: [String.fromCodePoint]
---*/

assert.throws(RangeError, function() {
  String.fromCodePoint(-1);
});

assert.throws(RangeError, function() {
  String.fromCodePoint(1, -1);
});

assert.throws(RangeError, function() {
  String.fromCodePoint(1114112);
});

assert.throws(RangeError, function() {
  String.fromCodePoint(Infinity);
});
