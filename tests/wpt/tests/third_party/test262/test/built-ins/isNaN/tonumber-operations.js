// Copyright (C) 2016 The V8 Project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-isnan-number
description: >
  number argument is converted by ToNumber
info: |
  isNaN (number)

  1. Let num be ? ToNumber(number).
  2. If num is NaN, return true.
  3. Otherwise, return false.
---*/

assert.sameValue(isNaN("0"), false, "'0'");
assert.sameValue(isNaN(""), false, "the empty string");
assert.sameValue(isNaN("Infinity"), false, "'Infinity'");
assert.sameValue(isNaN("this is not a number"), true, "string");
assert.sameValue(isNaN(true), false, "true");
assert.sameValue(isNaN(false), false, "false");
assert.sameValue(isNaN([1]), false, "Object [1]");
assert.sameValue(isNaN([Infinity]), false, "Object [Infinity]");
assert.sameValue(isNaN([NaN]), true, "Object [NaN]");
assert.sameValue(isNaN(null), false, "null");
assert.sameValue(isNaN(undefined), true, "undefined");
assert.sameValue(isNaN(), true, "no arg");
