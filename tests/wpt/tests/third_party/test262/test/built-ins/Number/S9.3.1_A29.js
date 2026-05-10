// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: "The MV of HexDigit ::: d or of HexDigit ::: D is 13"
es5id: 9.3.1_A29
description: >
    Compare Number('0xD'), Number('0XD'), Number('0xd') and
    Number('0Xd') with 13
---*/
assert.sameValue(+("0xd"), 13, 'The value of `+("0xd")` is expected to be 13');
assert.sameValue(Number("0xD"), 13, 'Number("0xD") must return 13');
assert.sameValue(Number("0Xd"), 13, 'Number("0Xd") must return 13');
assert.sameValue(Number("0XD"), 13, 'Number("0XD") must return 13');
