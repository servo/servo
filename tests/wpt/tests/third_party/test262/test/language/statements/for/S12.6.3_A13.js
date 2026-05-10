// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    VariableDeclaration in "var VariableDeclarationListNoIn" of for
    IterationStatement is allowed
es5id: 12.6.3_A13
description: Declaring variable in "for" ExpressionNoIn
---*/

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
try {
	index = index;
} catch (e) {
	throw new Test262Error('#1: VariableDeclaration in "var VariableDeclarationListNoIn" of for IterationStatement is allowed');
}
//
//////////////////////////////////////////////////////////////////////////////


for(var index=0; index<6; index++) {
	;
}
