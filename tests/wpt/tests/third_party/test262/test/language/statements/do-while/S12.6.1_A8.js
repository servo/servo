// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: "\"continue\" statement within a \"do-while\" Statement is allowed"
es5id: 12.6.1_A8
description: Using eval
---*/

var __evaluated;
var __condition = 0, __odds=0;

__evaluated = eval("do { __condition++; if (((''+__condition/2).split('.')).length>1) continue; __odds++;} while(__condition < 10)");

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (__odds !== 5) {
	throw new Test262Error('#1: __odds === 5. Actual:  __odds ==='+ __odds  );
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#2
if (__evaluated !== 4) {
	throw new Test262Error('#2: __evaluated === 4. Actual:  __evaluated ==='+ __evaluated  );
}
//
//////////////////////////////////////////////////////////////////////////////
