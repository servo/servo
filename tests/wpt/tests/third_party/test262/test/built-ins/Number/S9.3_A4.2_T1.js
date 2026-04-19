// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Result of number conversion from number value equals to the input
    argument (no conversion)
es5id: 9.3_A4.2_T1
description: >
    Number.NaN, +0, -0, Number.POSITIVE_INFINITY,
    Number.NEGATIVE_INFINITY,  Number.MAX_VALUE and Number.MIN_VALUE
    convert to Number by explicit transformation
---*/

// CHECK#1
assert.sameValue(Number(NaN), NaN, 'Number(true) returns NaN');

assert.sameValue(Number(+0), +0, 'Number(+0) must return +0');
assert.sameValue(Number(-0), -0, 'Number(-0) must return -0');

assert.sameValue(
  Number(Number.POSITIVE_INFINITY),
  Number.POSITIVE_INFINITY,
  'Number(Number.POSITIVE_INFINITY) returns Number.POSITIVE_INFINITY'
);

assert.sameValue(
  Number(Number.NEGATIVE_INFINITY),
  Number.NEGATIVE_INFINITY,
  'Number(Number.NEGATIVE_INFINITY) returns Number.NEGATIVE_INFINITY'
);

assert.sameValue(Number(Number.MAX_VALUE), Number.MAX_VALUE, 'Number(Number.MAX_VALUE) returns Number.MAX_VALUE');
assert.sameValue(Number(Number.MIN_VALUE), Number.MIN_VALUE, 'Number(Number.MIN_VALUE) returns Number.MIN_VALUE');
