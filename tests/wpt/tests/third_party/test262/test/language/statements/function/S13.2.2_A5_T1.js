// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    When the [[Construct]] property for a Function object F is called:
    A new native ECMAScript object is created.
    Invoke the [[Call]] property of F, providing native ECMAScript object just created as the this value and
    providing the argument list passed into [[Construct]] as the argument values
es5id: 13.2.2_A5_T1
description: Declaring a function with "function __FACTORY(arg1, arg2)"
---*/

var __VOLUME, __RED, __ID, __BOTTOM, __TOP, __LEFT, color, bottom, left, __device;

__VOLUME=8;
__RED="red";
__ID=12342;
__BOTTOM=1.1;
__TOP=0.1;
__LEFT=0.5;


function __FACTORY(arg1, arg2){
	this.volume=__VOLUME;
	color=__RED;
	this.id=arg1;
	bottom=arg2;
	this.top=arguments[2];
	left=arguments[3];
};

__device = new __FACTORY(__ID, __BOTTOM, __TOP, __LEFT);

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (__device.color !== undefined) {
	throw new Test262Error('#1: __device.color === undefined. Actual: __device.color ==='+__device.color);
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#2
if (__device.volume !== __VOLUME) {
	throw new Test262Error('#2: __device.volume === __VOLUME. Actual: __device.volume ==='+__device.volume);
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#3
if (__device.bottom !== undefined) {
	throw new Test262Error('#3: __device.bottom === undefined. Actual: __device.bottom ==='+__device.bottom);
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#4
if (__device.id !== __ID) {
	throw new Test262Error('#4: __device.id === __ID. Actual: __device.id ==='+__device.id);
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#5
if (__device.left !== undefined) {
	throw new Test262Error('#5: __device.left === undefined. Actual: __device.left ==='+__device.left);
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#6
if (__device.top !== __TOP) {
	throw new Test262Error('#6: __device.top === __TOP. Actual: __device.top ==='+__device.top);
}
//
//////////////////////////////////////////////////////////////////////////////
