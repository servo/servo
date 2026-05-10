// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    While evaluating The production IterationStatement: "while ( Expression )
    Statement", Expression is evaluated first
es5id: 12.6.2_A2
description: Evaluating Statement with error Expression
---*/

try {
	while ((function(){throw 1})()) __in__while = "reached"; 
	throw new Test262Error('#1: \'while ((function(){throw 1})()) __in__while = "reached"\' lead to throwing exception');
} catch (e) {
	if (e !== 1) {
		throw new Test262Error('#1: Exception === 1. Actual:  Exception ==='+e);
	}
}

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (typeof __in__while !== "undefined") {
	throw new Test262Error('#1.1: typeof __in__while === "undefined". Actual: typeof __in__while ==='+typeof __in__while);
}
//
//////////////////////////////////////////////////////////////////////////////
