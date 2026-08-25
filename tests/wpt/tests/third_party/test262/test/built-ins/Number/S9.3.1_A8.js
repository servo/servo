// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The MV of StrUnsignedDecimalLiteral::: DecimalDigits. ExponentPart
    is the MV of DecimalDigits times 10<sup><small>e</small></sup> , where e is the MV of ExponentPart
es5id: 9.3.1_A8
description: >
    Compare Number('1234e5') and Number('1234.e5') with
    Number('1234')*1e5
---*/
assert.sameValue(Number("1234e5"), 123400000, 'Number("1234e5") must return 123400000');
assert.sameValue(Number("1234.e5"), 123400000, 'Number("1234.e5") must return 123400000');
