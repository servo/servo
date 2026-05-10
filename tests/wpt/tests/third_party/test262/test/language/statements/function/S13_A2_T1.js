// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: function must be evaluated inside the expression
es5id: 13_A2_T1
description: Defining function body with "return arg"
---*/

var x = (function __func(arg){return arg})(1);

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (x !== 1) {
	throw new Test262Error('#1: x === 1. Actual: x ==='+x);
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
