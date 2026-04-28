// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    toString: If radix is the number 10 or undefined, then this
    number value is given as an argument to the ToString operator.
    the resulting string value is returned
es5id: 15.7.4.2_A1_T02
description: radix is 10
---*/
assert.sameValue(Number.prototype.toString(10), "0", 'Number.prototype.toString(10) must return "0"');
assert.sameValue((new Number()).toString(10), "0", '(new Number()).toString(10) must return "0"');
assert.sameValue((new Number(0)).toString(10), "0", '(new Number(0)).toString(10) must return "0"');
assert.sameValue((new Number(-1)).toString(10), "-1", '(new Number(-1)).toString(10) must return "-1"');
assert.sameValue((new Number(1)).toString(10), "1", '(new Number(1)).toString(10) must return "1"');

assert.sameValue(
  (new Number(Number.NaN)).toString(10),
  "NaN",
  '(new Number(Number.NaN)).toString(10) must return "NaN"'
);

assert.sameValue(
  (new Number(Number.POSITIVE_INFINITY)).toString(10),
  "Infinity",
  '(new Number(Number.POSITIVE_INFINITY)).toString(10) must return "Infinity"'
);

assert.sameValue(
  (new Number(Number.NEGATIVE_INFINITY)).toString(10),
  "-Infinity",
  '(new Number(Number.NEGATIVE_INFINITY)).toString(10) must return "-Infinity"'
);
