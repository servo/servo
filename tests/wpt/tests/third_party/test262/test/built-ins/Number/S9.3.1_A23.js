// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: "The MV of DecimalDigit ::: 7 or of HexDigit ::: 7 is 7"
es5id: 9.3.1_A23
description: Compare Number('0x7') and Number('0X7') with 7
---*/
assert.sameValue(Number("7"), 7, 'Number("7") must return 7');
assert.sameValue(Number("0x7"), 7, 'Number("0x7") must return 7');
assert.sameValue(+("0X7"), 7, 'The value of `+("0X7")` is expected to be 7');
