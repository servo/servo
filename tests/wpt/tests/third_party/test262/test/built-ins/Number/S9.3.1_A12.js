// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The MV of StrUnsignedDecimalLiteral::: DecimalDigits ExponentPart
    is the MV of DecimalDigits times 10<sup><small>e</small></sup>, where e is the MV of ExponentPart
es5id: 9.3.1_A12
description: >
    Compare Number('12345e6') with +('12345')*1e1,  and
    Number('12345e-6') !== Number('12345')*1e-6
---*/
assert.sameValue(Number("12345e6"), 12345000000, 'Number("12345e6") must return 12345000000');
assert.sameValue(Number("12345e-6"), 0.012345, 'Number("12345e-6") must return 0.012345');
