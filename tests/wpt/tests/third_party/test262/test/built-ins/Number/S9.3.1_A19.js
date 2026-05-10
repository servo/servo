// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: "The MV of DecimalDigit ::: 3 or of HexDigit ::: 3 is 3"
es5id: 9.3.1_A19
description: Compare Number('0x3') and Number('0X3') with 3
---*/
assert.sameValue(Number("3"), 3, 'Number("3") must return 3');
assert.sameValue(+("0x3"), 3, 'The value of `+("0x3")` is expected to be 3');
assert.sameValue(Number("0X3"), 3, 'Number("0X3") must return 3');
