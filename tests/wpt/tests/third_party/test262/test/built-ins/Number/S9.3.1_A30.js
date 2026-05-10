// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: "The MV of HexDigit ::: e or of HexDigit ::: E is 14"
es5id: 9.3.1_A30
description: >
    Compare Number('0xE'), Number('0XE'), Number('0xe') and
    Number('0Xe') with 14
---*/
assert.sameValue(Number("0xe"), 14, 'Number("0xe") must return 14');
assert.sameValue(Number("0xE"), 14, 'Number("0xE") must return 14');
assert.sameValue(Number("0Xe"), 14, 'Number("0Xe") must return 14');
assert.sameValue(+("0XE"), 14, 'The value of `+("0XE")` is expected to be 14');
