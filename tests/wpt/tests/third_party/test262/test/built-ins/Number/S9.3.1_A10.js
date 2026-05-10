// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The MV of StrUnsignedDecimalLiteral:::. DecimalDigits is the
    MV of DecimalDigits times 10<sup><small>-n</small></sup>, where n is the
    number of characters in DecimalDigits
es5id: 9.3.1_A10
description: Compare Number('.12345') with +('12345')*1e-5
---*/
assert.sameValue(+('12345')*1e-5, 0.12345);
