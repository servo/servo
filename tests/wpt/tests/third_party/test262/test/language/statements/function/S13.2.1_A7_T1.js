// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    When the [[Call]] property for a Function object F is called, the following steps are taken:
    2. Evaluate F's FunctionBody;
    if Result.type is returned  then Result.value is returned too
es5id: 13.2.1_A7_T1
description: Returning null. Declaring a function with "function __func()"
---*/

function __func(){
    var x = null;
    return x;
}

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
try{
    var x=__func();
} catch(e){
    throw new Test262Error('#1: var x=__func() does not lead to throwing exception. Actual: exception is '+e);
}
//
//////////////////////////////////////////////////////////////////////////////
