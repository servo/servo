// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Objects as arguments are passed by reference
es5id: 13.2.1_A4_T1
description: >
    Adding new number property to a function argument within the
    function body,  where explicit argument is an object defined with
    "var __obj={}"
---*/

function __func(__arg){
    __arg.foo=7;
}

var __obj={};

__func(__obj);

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (__obj.foo !== 7) {
	throw new Test262Error('#1: __obj.foo === 7. Actual: __obj.foo ==='+__obj.foo);
}
//
//////////////////////////////////////////////////////////////////////////////
