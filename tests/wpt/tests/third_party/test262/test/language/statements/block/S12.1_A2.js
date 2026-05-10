// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The production StatementList  Statement is evaluated as follows
    1. Evaluate Statement.
    2. If an exception was thrown, return (throw, V, empty) where V is the exception
es5id: 12.1_A2
description: Throwing exception within a Block
---*/

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
assert.throws(ReferenceError, function() {
	{
		x();
	}
});

//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#2
try {
    throw "catchme";	
    throw new Test262Error('#2: throw "catchme" lead to throwing exception');
} catch (e) {
	if (e!=="catchme") {
		throw new Test262Error('#2.1: Exception === "catchme". Actual:  Exception ==='+ e  );
	}
}

//
//////////////////////////////////////////////////////////////////////////////
