// Copyright (C) 2016 The V8 Project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-number.prototype.toexponential
description: >
  ToInteger(fractionDigits operations)
info: |
  Number.prototype.toExponential ( fractionDigits )

  1. Let x be ? thisNumberValue(this value).
  2. Let f be ? ToInteger(fractionDigits).
  [...]
---*/

assert.sameValue((123.456).toExponential(0.1), "1e+2", "0.1");
assert.sameValue((123.456).toExponential(-0.1), "1e+2", "-0.1");
assert.sameValue((123.456).toExponential(0.9), "1e+2", "0.9");
assert.sameValue((123.456).toExponential(-0.9), "1e+2", "-0.9");

assert.sameValue((123.456).toExponential(false), "1e+2", "false");
assert.sameValue((123.456).toExponential(true), "1.2e+2", "true");

assert.sameValue((123.456).toExponential(NaN), "1e+2", "NaN");
assert.sameValue((123.456).toExponential(null), "1e+2", "null");

assert.sameValue((123.456).toExponential("2"), "1.23e+2", "string");
assert.sameValue((123.456).toExponential(""), "1e+2", "the empty string");

assert.sameValue((123.456).toExponential([]), "1e+2", "[]");
assert.sameValue((123.456).toExponential([2]), "1.23e+2", "[2]");

assert.sameValue((0).toExponential(undefined), "0e+0", "undefined");
assert.sameValue((0).toExponential(), "0e+0", "no arg");
