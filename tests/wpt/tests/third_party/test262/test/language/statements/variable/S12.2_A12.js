// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: VariableDeclaration within "do-while" loop is allowed
es5id: 12.2_A12
description: Declaring variable within "do-while" statement
---*/

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
try {
	x=x;
} catch (e) {
	throw new Test262Error('#1: Declaration variable inside "do-while" statement is admitted');
}
//
//////////////////////////////////////////////////////////////////////////////

do var x; while (false);
