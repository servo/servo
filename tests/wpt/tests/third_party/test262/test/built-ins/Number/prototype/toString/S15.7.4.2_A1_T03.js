// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    toString: If radix is the number 10 or undefined, then this
    number value is given as an argument to the ToString operator.
    the resulting string value is returned
es5id: 15.7.4.2_A1_T03
description: radix is undefined value
---*/
assert.sameValue(Number.prototype.toString(undefined), "0", 'Number.prototype.toString(undefined) must return "0"');
assert.sameValue((new Number()).toString(undefined), "0", '(new Number()).toString(undefined) must return "0"');
assert.sameValue((new Number(0)).toString(undefined), "0", '(new Number(0)).toString(undefined) must return "0"');
assert.sameValue((new Number(-1)).toString(undefined), "-1", '(new Number(-1)).toString(undefined) must return "-1"');
assert.sameValue((new Number(1)).toString(undefined), "1", '(new Number(1)).toString(undefined) must return "1"');

assert.sameValue(
  (new Number(Number.NaN)).toString(undefined),
  "NaN",
  '(new Number(Number.NaN)).toString(undefined) must return "NaN"'
);

assert.sameValue(
  (new Number(Number.POSITIVE_INFINITY)).toString(undefined),
  "Infinity",
  '(new Number(Number.POSITIVE_INFINITY)).toString(undefined) must return "Infinity"'
);

assert.sameValue(
  (new Number(Number.NEGATIVE_INFINITY)).toString(undefined),
  "-Infinity",
  '(new Number(Number.NEGATIVE_INFINITY)).toString(undefined) must return "-Infinity"'
);
