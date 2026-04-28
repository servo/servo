// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: "The MV of DecimalDigit ::: 5 or of HexDigit ::: 5 is 5"
es5id: 9.3.1_A21
description: Compare Number('0x5') and Number('0X5') with 5
---*/
assert.sameValue(+("5"), 5, 'The value of `+("5")` is expected to be 5');
assert.sameValue(Number("0x5"), 5, 'Number("0x5") must return 5');
assert.sameValue(Number("0X5"), 5, 'Number("0X5") must return 5');
