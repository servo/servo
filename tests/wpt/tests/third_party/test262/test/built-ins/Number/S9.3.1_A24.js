// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: "The MV of DecimalDigit ::: 8 or of HexDigit ::: 8 is 8"
es5id: 9.3.1_A24
description: Compare Number('0x8') and Number('0X8') with 8
---*/
assert.sameValue(+("8"), 8, 'The value of `+("8")` is expected to be 8');
assert.sameValue(Number("0x8"), 8, 'Number("0x8") must return 8');
assert.sameValue(Number("0X8"), 8, 'Number("0X8") must return 8');
