// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The production FunctionDeclaration: "function Identifier (
    FormalParameterList_opt ) { FunctionBody }" is processed by function
    declarations
es5id: 13_A4_T2
description: >
    Declaring a function that uses prefix increment operator within
    its "return" Expression
---*/

function __func(arg){return ++arg;};

//////////////////////////////////////////////////////////////////////////////
//CHECK#1 
if (typeof __func !== "function") {
	throw new Test262Error('#1: typeof __func === "function". Actual: typeof __func ==='+typeof __func);
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#2
if (__func(1) !== 2) {
	throw new Test262Error('#2: __func(1) === 2. Actual: __func(1) ==='+__func(1));
}
//
//////////////////////////////////////////////////////////////////////////////
