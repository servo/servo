// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: "The MV of DecimalDigit ::: 0 or of HexDigit ::: 0 is 0"
es5id: 9.3.1_A16
description: Compare Number('0x0') and Number('0X0') with 0
---*/
assert.sameValue(Number("0"), 0, 'Number("0") must return 0');
assert.sameValue(+("0x0"), 0, 'The value of `+("0x0")` is expected to be 0');
assert.sameValue(Number("0X0"), 0, 'Number("0X0") must return 0');
