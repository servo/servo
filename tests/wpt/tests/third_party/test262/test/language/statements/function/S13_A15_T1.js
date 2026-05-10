// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: "''arguments'' variable overrides ActivationObject.arguments"
es5id: 13_A15_T1
description: Declaring a function with "__func(arguments)"
flags: [noStrict]
---*/

function __func(arguments){
    return arguments;
};

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (__func(42) !== 42) {
	throw new Test262Error('#1: "arguments" variable overrides ActivationObject.arguments');
}
//
//////////////////////////////////////////////////////////////////////////////
