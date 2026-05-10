// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: LINE FEED (U+000A) may occur between any two tokens
esid: sec-line-terminators
es5id: 7.3_A1.1_T2
description: Insert real LINE FEED between tokens of var x=1
---*/

var
x
=
1;

assert.sameValue(x, 1);
