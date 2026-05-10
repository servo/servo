// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: "The MV of DecimalDigit ::: 2 or of HexDigit ::: 2 is 2"
es5id: 9.3.1_A18
description: Compare Number('0x2') and Number('0X2') with 2
---*/
assert.sameValue(+("2"), 2, 'The value of `+("2")` is expected to be 2');
assert.sameValue(Number("0x2"), 2, 'Number("0x2") must return 2');
assert.sameValue(Number("0X2"), 2, 'Number("0X2") must return 2');
