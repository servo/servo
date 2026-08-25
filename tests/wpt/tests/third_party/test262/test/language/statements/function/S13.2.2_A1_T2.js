// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Since a function is an object, it might be set to [[Prototype]] property
    of a new created object through [[Construct]] property
es5id: 13.2.2_A1_T2
description: Declaring a function with "var __PROTO = function()"
---*/

var __MONSTER="monster";
var __PREDATOR="predator";

var __PROTO = function(){};

try{
    __PROTO.type=__MONSTER;
}
catch(e){
    throw new Test262Error('#0: __PROTO.type=__MONSTER does not lead to throwing exception')
}

var __FACTORY = function(){};

__FACTORY.prototype=__PROTO;

var __monster = new __FACTORY();

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (!(__PROTO.isPrototypeOf(__monster))) {
	throw new Test262Error('#1: __PROTO.isPrototypeOf(__monster) must be true');
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#2
if (__monster.type !==__MONSTER) {
	throw new Test262Error('#2: __monster.type ===__MONSTER. Actual: __monster.type ==='+__monster.type);
}
//
//////////////////////////////////////////////////////////////////////////////
