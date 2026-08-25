// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Number.prototype.valueOf() returns this number value
es5id: 15.7.4.4_A1_T01
description: Call without argument
---*/
assert.sameValue(Number.prototype.valueOf(), 0, 'Number.prototype.valueOf() must return 0');
assert.sameValue((new Number()).valueOf(), 0, '(new Number()).valueOf() must return 0');
assert.sameValue((new Number(0)).valueOf(), 0, '(new Number(0)).valueOf() must return 0');
assert.sameValue((new Number(-1)).valueOf(), -1, '(new Number(-1)).valueOf() must return -1');
assert.sameValue((new Number(1)).valueOf(), 1, '(new Number(1)).valueOf() must return 1');

assert.sameValue(
  new Number(NaN).valueOf(),
  NaN,
  'new Number(NaN).valueOf() returns NaN'
);

assert.sameValue(
  (new Number(Number.POSITIVE_INFINITY)).valueOf(),
  Number.POSITIVE_INFINITY,
  '(new Number(Number.POSITIVE_INFINITY)).valueOf() returns Number.POSITIVE_INFINITY'
);

assert.sameValue(
  (new Number(Number.NEGATIVE_INFINITY)).valueOf(),
  Number.NEGATIVE_INFINITY,
  '(new Number(Number.NEGATIVE_INFINITY)).valueOf() returns Number.NEGATIVE_INFINITY'
);
