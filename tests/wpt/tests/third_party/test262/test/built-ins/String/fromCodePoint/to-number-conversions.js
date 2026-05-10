// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 21.1.2.2
description: >
  Returns the String value with the code unit for the given coerced types.
info: |
  String.fromCodePoint ( ...codePoints )

  1. Let result be the empty String.
  2. For each element next of codePoints, do
    a. Let nextCP be ? ToNumber(next).
    b. If nextCP is not an integral Number, throw a RangeError exception.
    c. If ℝ(nextCP) < 0 or ℝ(nextCP) > 0x10FFFF, throw a RangeError exception.
    d. Set result to the string-concatenation of result and UTF16EncodeCodePoint(ℝ(nextCP)).
  3. Assert: If codePoints is empty, then result is the empty String.
  4. Return result.
  
  Ref: 7.1.3 ToNumber ( argument )
features: [String.fromCodePoint]
---*/

assert.sameValue(String.fromCodePoint(null), '\x00');
assert.sameValue(String.fromCodePoint(false), '\x00');
assert.sameValue(String.fromCodePoint(true), '\x01');
assert.sameValue(String.fromCodePoint('42'), '\x2A');
assert.sameValue(String.fromCodePoint('042'), '\x2A');
assert.sameValue(
  String.fromCodePoint({
    valueOf: function() {
      return 31;
    }
  }),
  '\x1F'
);
