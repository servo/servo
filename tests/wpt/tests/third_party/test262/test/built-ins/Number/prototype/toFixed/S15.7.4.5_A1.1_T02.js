// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Step 1: Let f be ToInteger(fractionDigits). (If fractionDigits
    is undefined, this step produces the value 0)
es5id: 15.7.4.5_A1.1_T02
description: calling on Number object
---*/
assert.sameValue((new Number(1)).toFixed(), "1", '(new Number(1)).toFixed() must return "1"');
assert.sameValue((new Number(1)).toFixed(0), "1", '(new Number(1)).toFixed(0) must return "1"');
assert.sameValue((new Number(1)).toFixed(1), "1.0", '(new Number(1)).toFixed(1) must return "1.0"');
assert.sameValue((new Number(1)).toFixed(1.1), "1.0", '(new Number(1)).toFixed(1.1) must return "1.0"');
assert.sameValue((new Number(1)).toFixed(0.9), "1", '(new Number(1)).toFixed(0.9) must return "1"');
assert.sameValue((new Number(1)).toFixed("1"), "1.0", '(new Number(1)).toFixed("1") must return "1.0"');
assert.sameValue((new Number(1)).toFixed("1.1"), "1.0", '(new Number(1)).toFixed("1.1") must return "1.0"');
assert.sameValue((new Number(1)).toFixed("0.9"), "1", '(new Number(1)).toFixed("0.9") must return "1"');
assert.sameValue((new Number(1)).toFixed(Number.NaN), "1", '(new Number(1)).toFixed(Number.NaN) must return "1"');

assert.sameValue(
  (new Number(1)).toFixed("some string"),
  "1",
  '(new Number(1)).toFixed("some string") must return "1"'
);

try {
  assert.sameValue((new Number(1)).toFixed(-0.1), "1", '(new Number(1)).toFixed(-0.1) must return "1"');
}
catch (e) {
  throw new Test262Error('#10: (new Number(1)).toFixed(-0.1) should not throw ' + e);
}
