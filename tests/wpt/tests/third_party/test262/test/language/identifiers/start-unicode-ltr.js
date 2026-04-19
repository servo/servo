// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: "IdentifierPart :: IdentifierStart"
es5id: 7.6_A2.1_T1
description: "IdentifierStart :: UnicodeLetter"
---*/

var x = 1;
assert.sameValue(x, 1);

var xx = 2;
assert.sameValue(xx, 2);

var x$ = 3;
assert.sameValue(x$, 3);

var x_ = 4;
assert.sameValue(x_, 4);
