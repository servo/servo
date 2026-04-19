// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Step 1: Let f be ToInteger(fractionDigits). (If fractionDigits
    is undefined, this step produces the value 0)
es5id: 15.7.4.5_A1.1_T01
description: calling on Number prototype object
---*/
assert.sameValue(Number.prototype.toFixed(), "0", 'Number.prototype.toFixed() must return "0"');
assert.sameValue(Number.prototype.toFixed(0), "0", 'Number.prototype.toFixed(0) must return "0"');
assert.sameValue(Number.prototype.toFixed(1), "0.0", 'Number.prototype.toFixed(1) must return "0.0"');
assert.sameValue(Number.prototype.toFixed(1.1), "0.0", 'Number.prototype.toFixed(1.1) must return "0.0"');
assert.sameValue(Number.prototype.toFixed(0.9), "0", 'Number.prototype.toFixed(0.9) must return "0"');
assert.sameValue(Number.prototype.toFixed("1"), "0.0", 'Number.prototype.toFixed("1") must return "0.0"');
assert.sameValue(Number.prototype.toFixed("1.1"), "0.0", 'Number.prototype.toFixed("1.1") must return "0.0"');
assert.sameValue(Number.prototype.toFixed("0.9"), "0", 'Number.prototype.toFixed("0.9") must return "0"');
assert.sameValue(Number.prototype.toFixed(Number.NaN), "0", 'Number.prototype.toFixed(Number.NaN) must return "0"');

assert.sameValue(
  Number.prototype.toFixed("some string"),
  "0",
  'Number.prototype.toFixed("some string") must return "0"'
);

try {
  assert.sameValue(Number.prototype.toFixed(-0.1), "0", 'Number.prototype.toFixed(-0.1) must return "0"');
}
catch (e) {
  throw new Test262Error('#10: Number.prototype.toFixed(-0.1) should not throw ' + e);
}
