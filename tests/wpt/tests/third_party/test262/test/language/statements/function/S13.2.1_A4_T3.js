// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Objects as arguments are passed by reference
es5id: 13.2.1_A4_T3
description: >
    Adding new number property to a function argument within the
    function body,  where array element "arguments[0]" is an object
    defined with "__obj={}"
---*/

function __func(){
    arguments[0]["PI"]=3.14;
}

var __obj={};

__func(__obj);

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (__obj.PI !== 3.14) {
	throw new Test262Error('#1: __obj.PI === 3.14. Actual: __obj.PI ==='+__obj.PI);
}
//
//////////////////////////////////////////////////////////////////////////////
