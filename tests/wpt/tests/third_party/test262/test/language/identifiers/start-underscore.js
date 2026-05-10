// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: "IdentifierPart :: IdentifierStart"
es5id: 7.6_A2.1_T3
description: "IdentifierStart :: _"
---*/

var _ = 1;
assert.sameValue(_, 1);

var _x = 2;
assert.sameValue(_x, 2);

var _$ = 3;
assert.sameValue(_$, 3);

var __ = 4;
assert.sameValue(__, 4);
