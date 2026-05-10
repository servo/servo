// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Variables are created when the program is entered. Variables are initialised to "undefined"
    when created. A variable with an Initialiser is assigned the value of its AssignmentExpression when the
    VariableStatement is executed, not when the variable is created
es5id: 12.2_A1
description: Creating variables after entering the execution scope
---*/

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
try {
	__x = __x;
    __y = __x ? "good fellow" : "liar"; // __y assigned to "liar" since __x undefined
    __z = __z === __x ? 1 : 0; // __z assigned to 1 since both __x and __z are undefined
} catch (e) {
	throw new Test262Error('#1: Using declarated variable before it declaration is admitted');
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#2
assert.throws(ReferenceError, function() {
    __something__undefined = __something__undefined;
});
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#3
if ((__y !== "liar")&(__z !== 1)) {
	throw new Test262Error('#3: (__y === "liar") and (__z === 1). Actual:  __y ==='+__y+' and __z ==='+__z  );
}
//
//////////////////////////////////////////////////////////////////////////////

var __x, __y = true, __z = __y ? "smeagol" : "golum";

//////////////////////////////////////////////////////////////////////////////
//CHECK#4
if (!__y&!(__z = "smeagol")) {
	throw new Test262Error('#4: A variable with an Initialiser is assigned the value of its AssignmentExpression when the VariableStatement is executed');
}
//
//////////////////////////////////////////////////////////////////////////////
