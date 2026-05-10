// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    When the [[Call]] property for a Function object F is called, the following steps are taken:
    2. Evaluate F's FunctionBody;
    if Result.type is returned  then Result.value is returned too
es5id: 13.2.1_A7_T4
description: Returning boolean. Declaring a function with "function __func()"
---*/

function __func(){
    var x = true;
    return x;
}

//////////////////////////////////////////////////////////////////////////////
//CHECK#
try {
	x=x;
	throw new Test262Error('#0: "x=x" lead to throwing exception');
} catch (e) {
    if (e instanceof Test262Error) throw e;
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
try{
    var __x=__func()
} catch(e){
    throw new Test262Error('#1: var __x=__func() does not lead to throwing exception. Actual: exception is '+e);
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#2
if (!(__x)) {
	throw new Test262Error('#2: __x === true. Actual: __x ==='+__x);
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#3
try {
	x=x;
	throw new Test262Error('#3: "x=x" lead to throwing exception');
} catch (e) {
    if (e instanceof Test262Error) throw e;
}
//
//////////////////////////////////////////////////////////////////////////////
