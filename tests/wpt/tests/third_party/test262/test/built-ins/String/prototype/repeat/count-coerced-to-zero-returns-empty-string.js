// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 21.1.3.13
description: >
  If ToInteger(count) is zero, returns an empty String.
info: |
  21.1.3.13 String.prototype.repeat ( count )

  8. Let T be a String value that is made from n copies of S appended together.
  If n is 0, T is the empty String.
  9. Return T.
---*/

var str = 'ES2015';

assert.sameValue(str.repeat(NaN), '', 'str.repeat(NaN) returns ""');
assert.sameValue(str.repeat(null), '', 'str.repeat(null) returns ""');
assert.sameValue(str.repeat(undefined), '', 'str.repeat(undefined) returns ""');
assert.sameValue(str.repeat(false), '', 'str.repeat(false) returns ""');
assert.sameValue(str.repeat('0'), '', 'str.repeat("0") returns ""');
assert.sameValue(str.repeat(0.9), '', 'str.repeat(0.9) returns ""');
