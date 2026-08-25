// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    toString: If radix is the number 10 or undefined, then this
    number value is given as an argument to the ToString operator.
    the resulting string value is returned
es5id: 15.7.4.2_A1_T01
description: undefined radix
---*/
assert.sameValue(Number.prototype.toString(), "0", 'Number.prototype.toString() must return "0"');
assert.sameValue((new Number()).toString(), "0", '(new Number()).toString() must return "0"');
assert.sameValue((new Number(0)).toString(), "0", '(new Number(0)).toString() must return "0"');
assert.sameValue((new Number(-1)).toString(), "-1", '(new Number(-1)).toString() must return "-1"');
assert.sameValue((new Number(1)).toString(), "1", '(new Number(1)).toString() must return "1"');
assert.sameValue((new Number(Number.NaN)).toString(), "NaN", '(new Number(Number.NaN)).toString() must return "NaN"');

assert.sameValue(
  (new Number(Number.POSITIVE_INFINITY)).toString(),
  "Infinity",
  '(new Number(Number.POSITIVE_INFINITY)).toString() must return "Infinity"'
);

assert.sameValue(
  (new Number(Number.NEGATIVE_INFINITY)).toString(),
  "-Infinity",
  '(new Number(Number.NEGATIVE_INFINITY)).toString() must return "-Infinity"'
);
