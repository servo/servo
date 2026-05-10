// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Since a function is an object, it might be set to [[Prototype]] property of a new created object through [[Construct]] property,
    but [[call]] property must fail with TypeError error
es5id: 13.2.2_A2
description: Trying to [[call]] this function
---*/

var __PLANT="flower";
var __ROSE="rose";

function __PROTO(){};

try{
    __PROTO.type=__PLANT;
}
catch(e){
    throw new Test262Error('#0: __PROTO.type=__PLANT does not lead to throwing exception')
}

function __FACTORY(){};

__FACTORY.prototype=__PROTO;

var __rose = new __FACTORY();

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
try{
    __rose();
    throw new Test262Error('#1: __rose() lead to throwing exception');
} catch(e){
    if (!(e instanceof TypeError)) {
    	throw new Test262Error('#2: Exception Type is TypeError. Actual: exception ==='+e);
    }
}
//
//////////////////////////////////////////////////////////////////////////////
