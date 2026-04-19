// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The MV of StrDecimalLiteral::: - StrUnsignedDecimalLiteral is the negative
    of the MV of StrUnsignedDecimalLiteral. (the negative of this 0 is also 0)
es5id: 9.3.1_A5_T3
description: Compare Number('-' + 'any_number') with -Number('any_number')
---*/

function dynaString(s1, s2) {
  return String(s1) + String(s2);
}

assert.sameValue(Number(dynaString("-", "0")), -0, 'Number(dynaString("-", "0")) must return -Number("0")');

assert.sameValue(
  Number(dynaString("-Infi", "nity")),
  -Infinity
);

assert.sameValue(
  Number(dynaString("-12345", "67890")),
  -1234567890
);

assert.sameValue(
  Number(dynaString("-1234.", "5678")),
  -1234.5678
);

assert.sameValue(
  Number(dynaString("-1234.", "5678e90")),
  -1234.5678e90
);

assert.sameValue(
  Number(dynaString("-1234.", "5678E90")),
  -1234.5678E90
);

assert.sameValue(
  Number(dynaString("-1234.", "5678e-90")),
  -1234.5678e-90
);

assert.sameValue(
  Number(dynaString("-1234.", "5678E-90")),
  -1234.5678E-90
);

assert.sameValue(
  Number(dynaString("-Infi", "nity")),
  Number.NEGATIVE_INFINITY
);
