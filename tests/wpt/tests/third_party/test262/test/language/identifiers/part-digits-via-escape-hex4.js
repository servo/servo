// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Correct interpretation of DIGITS
es5id: 7.6_A4.3_T1
description: Identifier is $+ANY_DIGIT
---*/

var $\u0030 = 0;
assert.sameValue($0, 0);

var $\u0031 = 1;
assert.sameValue($1, 1);

var $\u0032 = 2;
assert.sameValue($2, 2);

var $\u0033 = 3;
assert.sameValue($3, 3);

var $\u0034 = 4;
assert.sameValue($4, 4);

var $\u0035 = 5;
assert.sameValue($5, 5);

var $\u0036 = 6;
assert.sameValue($6, 6);

var $\u0037 = 7;
assert.sameValue($7, 7);

var $\u0038 = 8;
assert.sameValue($8, 8);

var $\u0039 = 9;
assert.sameValue($9, 9);
