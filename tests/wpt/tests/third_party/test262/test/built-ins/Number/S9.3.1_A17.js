// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: "The MV of DecimalDigit ::: 1 or of HexDigit ::: 1 is 1"
es5id: 9.3.1_A17
description: Compare Number('0x1') and Number('0X1') with 1
---*/
assert.sameValue(Number("1"), 1, 'Number("1") must return 1');
assert.sameValue(Number("0x1"), 1, 'Number("0x1") must return 1');
assert.sameValue(+("0X1"), 1, 'The value of `+("0X1")` is expected to be 1');
