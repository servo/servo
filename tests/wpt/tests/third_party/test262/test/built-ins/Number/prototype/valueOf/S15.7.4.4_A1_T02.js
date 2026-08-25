// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Number.prototype.valueOf() returns this number value
es5id: 15.7.4.4_A1_T02
description: calling with argument
---*/
assert.sameValue(Number.prototype.valueOf("argument"), 0, 'Number.prototype.valueOf("argument") must return 0');
assert.sameValue((new Number()).valueOf("argument"), 0, '(new Number()).valueOf("argument") must return 0');
assert.sameValue((new Number(0)).valueOf("argument"), 0, '(new Number(0)).valueOf("argument") must return 0');
assert.sameValue((new Number(-1)).valueOf("argument"), -1, '(new Number(-1)).valueOf("argument") must return -1');
assert.sameValue((new Number(1)).valueOf("argument"), 1, '(new Number(1)).valueOf("argument") must return 1');

assert.sameValue(
  new Number(NaN).valueOf("argument"),
  NaN,
  'new Number(NaN).valueOf("argument") returns NaN'
);

assert.sameValue(
  (new Number(Number.POSITIVE_INFINITY)).valueOf("argument"),
  Number.POSITIVE_INFINITY,
  '(new Number(Number.POSITIVE_INFINITY)).valueOf("argument") returns Number.POSITIVE_INFINITY'
);

assert.sameValue(
  (new Number(Number.NEGATIVE_INFINITY)).valueOf("argument"),
  Number.NEGATIVE_INFINITY,
  '(new Number(Number.NEGATIVE_INFINITY)).valueOf("argument") returns Number.NEGATIVE_INFINITY'
);
