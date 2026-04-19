// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: "Step 4: If this number value is NaN, return the string \"NaN\""
es5id: 15.7.4.5_A1.3_T02
description: direct usage of NaN
---*/
assert.sameValue(Number.NaN.toFixed(), "NaN", 'Number.NaN.toFixed() must return "NaN"');
assert.sameValue(Number.NaN.toFixed(0), "NaN", 'Number.NaN.toFixed(0) must return "NaN"');
assert.sameValue(Number.NaN.toFixed(1), "NaN", 'Number.NaN.toFixed(1) must return "NaN"');
assert.sameValue(Number.NaN.toFixed(1.1), "NaN", 'Number.NaN.toFixed(1.1) must return "NaN"');
assert.sameValue(Number.NaN.toFixed(0.9), "NaN", 'Number.NaN.toFixed(0.9) must return "NaN"');
assert.sameValue(Number.NaN.toFixed("1"), "NaN", 'Number.NaN.toFixed("1") must return "NaN"');
assert.sameValue(Number.NaN.toFixed("1.1"), "NaN", 'Number.NaN.toFixed("1.1") must return "NaN"');
assert.sameValue(Number.NaN.toFixed("0.9"), "NaN", 'Number.NaN.toFixed("0.9") must return "NaN"');
assert.sameValue(Number.NaN.toFixed(Number.NaN), "NaN", 'Number.NaN.toFixed(Number.NaN) must return "NaN"');
assert.sameValue(Number.NaN.toFixed("some string"), "NaN", 'Number.NaN.toFixed("some string") must return "NaN"');

try {
  s = Number.NaN.toFixed(Number.POSITIVE_INFINITY);
  throw new Test262Error('#10: Number.NaN.toFixed(Number.POSITIVE_INFINITY) should throw RangeError, not return NaN');
}
catch (e) {
  assert(e instanceof RangeError, 'The result of evaluating (e instanceof RangeError) is expected to be true');
}
