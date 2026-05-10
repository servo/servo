// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: function must be evaluated inside the expression
es5id: 13_A2_T2
description: Defining function body with "return arg + arguments[1]"
---*/

var x = (function __func(arg){return arg + arguments[1]})(1,"1");

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (x !== "11") {
	throw new Test262Error('#1: x === "11". Actual: x ==='+x);
}

//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#2
if (typeof __func !== 'undefined') {
	throw new Test262Error('#2: typeof __func === \'undefined\'. Actual: typeof __func ==='+typeof __func);
}
//
//////////////////////////////////////////////////////////////////////////////
