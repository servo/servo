// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The MV of DecimalDigits ::: DecimalDigits DecimalDigit is
    (the MV of DecimalDigits times 10) plus the MV of DecimalDigit
es5id: 9.3.1_A13
description: Compare '12' with Number("1")*10+Number("2") and analogous
---*/
assert.sameValue(
  +("12"),
  12,
  'The value of `+("12")` is expected to be 12'
);

assert.sameValue(
  Number("123"),
  123,
  'Number("123") must return 123'
);

assert.sameValue(
  Number("1234"),
  1234,
  'Number("1234") must return 1234'
);
