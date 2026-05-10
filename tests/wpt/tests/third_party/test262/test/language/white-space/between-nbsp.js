// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: NO-BREAK SPACE (U+00A0) between any two tokens is allowed
es5id: 7.2_A1.5_T2
description: Insert real NO-BREAK SPACE between tokens of var x=1
---*/

 var x = 2 ;

assert.sameValue(x, 2);
