// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: "The MV of DecimalDigit ::: 9 or of HexDigit ::: 9 is 9"
es5id: 9.3.1_A25
description: Compare Number('0x9') and Number('0X9') with 9
---*/
assert.sameValue(Number("9"), 9, 'Number("9") must return 9');
assert.sameValue(+("0x9"), 9, 'The value of `+("0x9")` is expected to be 9');
assert.sameValue(Number("0X9"), 9, 'Number("0X9") must return 9');
