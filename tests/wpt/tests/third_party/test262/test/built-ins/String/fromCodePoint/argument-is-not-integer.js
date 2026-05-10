// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 21.1.2.2
description: >
  Throw a RangeError if an argument is not equal to its Integer representation.
info: |
  String.fromCodePoint ( ...codePoints )

  1. Let result be the empty String.
  2. For each element next of codePoints, do
    a. Let nextCP be ? ToNumber(next).
    b. If nextCP is not an integral Number, throw a RangeError exception.
  ...
features: [String.fromCodePoint]
---*/

assert.throws(RangeError, function() {
  String.fromCodePoint(3.14);
});

assert.throws(RangeError, function() {
  String.fromCodePoint(42, 3.14);
});

assert.throws(RangeError, function() {
  String.fromCodePoint('3.14');
});

// ToNumber(undefined) returns NaN.
assert.throws(RangeError, function() {
  String.fromCodePoint(undefined);
});

assert.throws(RangeError, function() {
  String.fromCodePoint('_1');
});

assert.throws(RangeError, function() {
  String.fromCodePoint('1a');
});
