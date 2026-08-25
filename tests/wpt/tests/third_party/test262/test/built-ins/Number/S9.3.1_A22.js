// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: "The MV of DecimalDigit ::: 6 or of HexDigit ::: 6 is 6"
es5id: 9.3.1_A22
description: Compare Number('0x6') and Number('0X6') with 6
---*/
assert.sameValue(Number("6"), 6, 'Number("6") must return 6');
assert.sameValue(+("0x6"), 6, 'The value of `+("0x6")` is expected to be 6');
assert.sameValue(Number("0X6"), 6, 'Number("0X6") must return 6');
