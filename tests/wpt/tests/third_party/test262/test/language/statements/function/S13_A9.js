// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Function can be passed as argument
es5id: 13_A9
description: Using function as argument of another function
---*/

function __func__INC(arg){return arg + 1;};
function __func__MULT(incrementator, arg, mult){ return incrementator(arg)*mult; };

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (__func__MULT(__func__INC, 2, 2) !== 6) {
	throw new Test262Error('#1: function  can be passed as argument');
}
//
//////////////////////////////////////////////////////////////////////////////
