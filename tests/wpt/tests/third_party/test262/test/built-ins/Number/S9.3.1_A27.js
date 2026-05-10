// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: "The MV of HexDigit ::: b or of HexDigit ::: B is 11"
es5id: 9.3.1_A27
description: >
    Compare Number('0xB'), Number('0XB'), Number('0xb') and
    Number('0Xb') with 11
---*/
assert.sameValue(Number("0xb"), 11, 'Number("0xb") must return 11');
assert.sameValue(Number("0xB"), 11, 'Number("0xB") must return 11');
assert.sameValue(+("0Xb"), 11, 'The value of `+("0Xb")` is expected to be 11');
assert.sameValue(Number("0XB"), 11, 'Number("0XB") must return 11');
