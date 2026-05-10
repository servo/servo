// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: "The MV of HexDigit ::: f or of HexDigit ::: F is 15"
es5id: 9.3.1_A31
description: >
    Compare Number('0xF'), Number('0XF'), Number('0xf') and
    Number('0Xf') with 15
---*/
assert.sameValue(Number("0xf"), 15, 'Number("0xf") must return 15');
assert.sameValue(Number("0xF"), 15, 'Number("0xF") must return 15');
assert.sameValue(+("0Xf"), 15, 'The value of `+("0Xf")` is expected to be 15');
assert.sameValue(Number("0XF"), 15, 'Number("0XF") must return 15');
