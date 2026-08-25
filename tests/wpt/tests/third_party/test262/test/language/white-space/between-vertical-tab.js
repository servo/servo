// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: VERTICAL TAB (U+000B) between any two tokens is allowed
es5id: 7.2_A1.2_T2
description: Insert real VERTICAL TAB between tokens of var x=1
---*/

varx=1;

assert.sameValue(x, 1);
