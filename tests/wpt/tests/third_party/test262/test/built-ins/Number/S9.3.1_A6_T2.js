// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The MV of StrUnsignedDecimalLiteral::: Infinity is 10<sup><small>10000</small></sup>
    (a value so large that it will round to <b><tt>+&infin;</tt></b>)
es5id: 9.3.1_A6_T2
description: >
    Compare Number('Infi'+'nity') with Number.POSITIVE_INFINITY,
    10e10000, 10E10000 and Number("10e10000")
---*/

function dynaString(s1, s2) {
  return String(s1) + String(s2);
}


assert.sameValue(
  Number(dynaString("Infi", "nity")),
  Number.POSITIVE_INFINITY,
  'Number(dynaString("Infi", "nity")) returns Number.POSITIVE_INFINITY'
);

assert.sameValue(
  Number(dynaString("Infi", "nity")),
  10e10000,
  'Number(dynaString("Infi", "nity")) must return 10e10000'
);

assert.sameValue(
  Number(dynaString("Infi", "nity")),
  10E10000,
  'Number(dynaString("Infi", "nity")) must return 10E10000'
);

assert.sameValue(
  Number(dynaString("Infi", "nity")),
  Number("10e10000"),
  'Number(dynaString("Infi", "nity")) must return the same value returned by Number("10e10000")'
);
