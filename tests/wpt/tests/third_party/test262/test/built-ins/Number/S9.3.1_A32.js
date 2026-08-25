// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Once the exact MV for a string numeric literal has been
    determined, it is then rounded to a value of the Number type with 20
    significant digits by replacing each significant digit after the 20th
    with a 0 digit or the number value
es5id: 9.3.1_A32
description: Use various long numbers, for example, 1234567890.1234567890
---*/
assert.sameValue(
  Number("1234567890.1234567890"),
  1234567890.1234567890,
  'Number("1234567890.1234567890") must return 1234567890.1234567890'
);

assert.sameValue(
  Number("1234567890.1234567890"),
  1234567890.1234567000,
  'Number("1234567890.1234567890") must return 1234567890.1234567000'
);

assert.notSameValue(
  +("1234567890.1234567890"),
  1234567890.123456,
  'The value of +("1234567890.1234567890") is not 1234567890.123456'
);

assert.sameValue(
  Number("0.12345678901234567890"),
  0.123456789012345678,
  'Number("0.12345678901234567890") must return 0.123456789012345678'
);

assert.sameValue(
  Number("00.12345678901234567890"),
  0.123456789012345678,
  'Number("00.12345678901234567890") must return 0.123456789012345678'
);
