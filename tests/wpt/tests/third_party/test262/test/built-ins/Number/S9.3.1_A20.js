// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: "The MV of DecimalDigit ::: 4 or of HexDigit ::: 4 is 4"
es5id: 9.3.1_A20
description: Compare Number('0x4') and Number('0X4') with 4
---*/
assert.sameValue(Number("4"), 4, 'Number("4") must return 4');
assert.sameValue(Number("0x4"), 4, 'Number("0x4") must return 4');
assert.sameValue(+("0X4"), 4, 'The value of `+("0X4")` is expected to be 4');
