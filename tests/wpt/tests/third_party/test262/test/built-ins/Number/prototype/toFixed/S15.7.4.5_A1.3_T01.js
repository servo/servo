// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: "Step 4: If this number value is NaN, return the string \"NaN\""
es5id: 15.7.4.5_A1.3_T01
description: NaN is computed by new Number("string")
---*/
assert.sameValue((new Number("a")).toFixed(), "NaN", '(new Number("a")).toFixed() must return "NaN"');
assert.sameValue((new Number("a")).toFixed(0), "NaN", '(new Number("a")).toFixed(0) must return "NaN"');
assert.sameValue((new Number("a")).toFixed(1), "NaN", '(new Number("a")).toFixed(1) must return "NaN"');
assert.sameValue((new Number("a")).toFixed(1.1), "NaN", '(new Number("a")).toFixed(1.1) must return "NaN"');
assert.sameValue((new Number("a")).toFixed(0.9), "NaN", '(new Number("a")).toFixed(0.9) must return "NaN"');
assert.sameValue((new Number("a")).toFixed("1"), "NaN", '(new Number("a")).toFixed("1") must return "NaN"');
assert.sameValue((new Number("a")).toFixed("1.1"), "NaN", '(new Number("a")).toFixed("1.1") must return "NaN"');
assert.sameValue((new Number("a")).toFixed("0.9"), "NaN", '(new Number("a")).toFixed("0.9") must return "NaN"');

assert.sameValue(
  (new Number("a")).toFixed(Number.NaN),
  "NaN",
  '(new Number("a")).toFixed(Number.NaN) must return "NaN"'
);

assert.sameValue(
  (new Number("a")).toFixed("some string"),
  "NaN",
  '(new Number("a")).toFixed("some string") must return "NaN"'
);

try {
  s = (new Number("a")).toFixed(Number.POSITIVE_INFINITY);
  throw new Test262Error('#10: (new Number("a")).toFixed(Number.POSITIVE_INFINITY) should throw RangeError, not return NaN');
}
catch (e) {
  assert(e instanceof RangeError, 'The result of evaluating (e instanceof RangeError) is expected to be true');
}
