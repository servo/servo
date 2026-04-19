// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: HORIZONTAL TAB (U+0009) between any two tokens is allowed
es5id: 7.2_A1.1_T2
description: Insert real HORIZONTAL TAB between tokens of var x=1
---*/

	var  x	=	1	;

assert.sameValue(x, 1);
