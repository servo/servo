// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The MV of StrUnsignedDecimalLiteral::: Infinity is 10<sup><small>10000</small></sup>
    (a value so large that it will round to <b><tt>+&infin;</tt></b>)
es5id: 9.3.1_A6_T1
description: >
    Compare Number('Infinity') with Number.POSITIVE_INFINITY,
    10e10000, 10E10000 and Number("10e10000")
---*/
assert.sameValue(Number("Infinity"), Number.POSITIVE_INFINITY, 'Number("Infinity") returns Number.POSITIVE_INFINITY');
assert.sameValue(Number("Infinity"), 10e10000, 'Number("Infinity") must return 10e10000');
assert.sameValue(Number("Infinity"), 10E10000, 'Number("Infinity") must return 10E10000');

assert.sameValue(
  Number("Infinity"),
  10e10000,
  'Number("Infinity") must return the same value returned by Number("10e10000")'
);
