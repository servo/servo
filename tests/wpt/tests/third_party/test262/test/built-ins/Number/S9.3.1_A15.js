// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The MV of SignedInteger ::: - DecimalDigits is the negative of the MV of
    DecimalDigits
es5id: 9.3.1_A15
description: Compare -Number('1234567890') with ('-1234567890')
---*/
assert.sameValue(
  +("-1234567890"),
  -1234567890,
  'The value of `+("-1234567890")` is expected to be -1234567890'
);
