// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Correct interpretation of DIGITS
es6id: 11.6
description: Identifier is $+ANY_DIGIT
---*/

var $0 = 0;
assert.sameValue($0, 0);

var $1 = 1;
assert.sameValue($1, 1);

var $2 = 2;
assert.sameValue($2, 2);

var $3 = 3;
assert.sameValue($3, 3);

var $4 = 4;
assert.sameValue($4, 4);

var $5 = 5;
assert.sameValue($5, 5);

var $6 = 6;
assert.sameValue($6, 6);

var $7 = 7;
assert.sameValue($7, 7);

var $8 = 8;
assert.sameValue($8, 8);

var $9 = 9;
assert.sameValue($9, 9);
