// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: "The MV of HexDigit ::: c or of HexDigit ::: C is 12"
es5id: 9.3.1_A28
description: >
    Compare Number('0xC'), Number('0XC'), Number('0xc') and
    Number('0Xc') with 12
---*/
assert.sameValue(Number("0xc"), 12, 'Number("0xc") must return 12');
assert.sameValue(+("0xC"), 12, 'The value of `+("0xC")` is expected to be 12');
assert.sameValue(Number("0Xc"), 12, 'Number("0Xc") must return 12');
assert.sameValue(Number("0XC"), 12, 'Number("0XC") must return 12');
