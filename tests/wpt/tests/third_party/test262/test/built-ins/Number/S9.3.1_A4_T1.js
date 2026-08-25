// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The MV of StrDecimalLiteral::: + StrUnsignedDecimalLiteral is the MV of
    StrUnsignedDecimalLiteral
es5id: 9.3.1_A4_T1
description: Compare Number('+any_number') with Number('any_number')
---*/
assert.sameValue(Number("+0"), Number("0"), 'Number("+0") must return the same value returned by Number("0")');

assert.sameValue(
  Number("+Infinity"),
  Infinity
);

assert.sameValue(
  Number("+1234.5678"),
  1234.5678
);

assert.sameValue(
  Number("+1234.5678e90"),
  1234.5678e90
);

assert.sameValue(
  Number("+1234.5678E90"),
  1234.5678E90
);

assert.sameValue(
  Number("+1234.5678e-90"),
  1234.5678e-90
);

assert.sameValue(
  Number("+1234.5678E-90"),
  1234.5678E-90
);
