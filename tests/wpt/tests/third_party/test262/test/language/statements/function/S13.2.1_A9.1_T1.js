// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    When the [[Call]] property for a Function object is called,
    the body is evaluated and if evaluation result has type "normal", then "undefined" is returned
es5id: 13.2.1_A9.1_T1
description: >
    Declaring a function with "function __func()" and no "return" in
    the function body
---*/

var x;

function __func(){
    x = true;
}

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (__func() !== undefined) {
	throw new Test262Error('#1: __func() === undefined. Actual: __func() ==='+__func());
};
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#2
if (!x) {
	throw new Test262Error('#2: x === true. Actual: x === '+x);
}
//
//////////////////////////////////////////////////////////////////////////////
