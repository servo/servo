// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    When the [[Construct]] property for a Function object F is called:
    A new native ECMAScript object is created.
    Invoke the [[Call]] property of F, providing just created native ECMAScript object as the this value and providing the argument
    list passed into [[Construct]] as the argument values.
    If Type( [[Call]] returned) is an Object then return this just as obtained object
es5id: 13.2.2_A7_T2
description: Declaring a "function as function __func (arg)"
---*/

var __FRST, __SCND, __func, __obj__;

__FRST="one";
__SCND="two";

__func = function(arg1, arg2){
	this.first=arg1;
	var __obj={second:arg2};
    return __obj;
	
};

__obj__ = new __func(__FRST, __SCND);

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (__obj__.first !== undefined) {
	throw new Test262Error('#1: __obj__.first === undefined. Actual: __obj__.first==='+__obj__.first);
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#2
if (__obj__.second !== __SCND) {
	throw new Test262Error('#2: __obj__.second === __SCND. Actual: __obj__.second ==='+__obj__.second);
}
//
//////////////////////////////////////////////////////////////////////////////
