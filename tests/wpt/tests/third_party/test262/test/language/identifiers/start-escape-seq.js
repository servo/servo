// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: "IdentifierPart :: IdentifierStart"
es5id: 7.6_A2.1_T4
description: "IdentifierStart :: \\UnicodeEscapeSequence"
---*/

var \u0078 = 1;
assert.sameValue(x, 1);

var \u0078\u0078 = 2;
assert.sameValue(xx, 2);

var \u0024 = 3;
assert.sameValue($, 3);

var \u0024x = 4;
assert.sameValue($x, 4);

var \u0024\u0024 = 5;
assert.sameValue($$, 5);

var \u0024_ = 6;
assert.sameValue($_, 6);

var \u005F = 7;
assert.sameValue(_, 7);

var \u005Fx = 8;
assert.sameValue(_x, 8);

var \u005F$ = 9;
assert.sameValue(_$, 9);

var \u005F\u005F = 10;
assert.sameValue(__, 10);
