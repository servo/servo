// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: "The MV of HexDigit ::: a or of HexDigit ::: A is 10"
es5id: 9.3.1_A26
description: >
    Compare Number('0xA'), Number('0XA'), Number('0xa') and
    Number('0Xa') with 10
---*/
assert.sameValue(Number("0xa"), 10, 'Number("0xa") must return 10');
assert.sameValue(Number("0xA"), 10, 'Number("0xA") must return 10');
assert.sameValue(Number("0Xa"), 10, 'Number("0Xa") must return 10');
assert.sameValue(+("0XA"), 10, 'The value of `+("0XA")` is expected to be 10');
