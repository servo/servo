// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Correct interpretation of DIGITS
es6id: 11.6
description: Identifier is $+ANY_DIGIT
---*/

var $\u{30} = 0;
assert.sameValue($0, 0);

var $\u{31} = 1;
assert.sameValue($1, 1);

var $\u{32} = 2;
assert.sameValue($2, 2);

var $\u{33} = 3;
assert.sameValue($3, 3);

var $\u{34} = 4;
assert.sameValue($4, 4);

var $\u{35} = 5;
assert.sameValue($5, 5);

var $\u{36} = 6;
assert.sameValue($6, 6);

var $\u{37} = 7;
assert.sameValue($7, 7);

var $\u{38} = 8;
assert.sameValue($8, 8);

var $\u{39} = 9;
assert.sameValue($9, 9);
