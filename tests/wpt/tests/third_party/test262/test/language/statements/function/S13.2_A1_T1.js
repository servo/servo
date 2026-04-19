// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: A "prototype" property is automatically created for every function
es5id: 13.2_A1_T1
description: Using "function __func(){}" as a FunctionDeclaration
---*/

function __func(){};

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (__func.prototype === undefined) {
	throw new Test262Error('#1: __func.prototype !== undefined');
}
//
//////////////////////////////////////////////////////////////////////////////
