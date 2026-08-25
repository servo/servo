// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Since arguments property has attribute { DontDelete }, only its elements
    can be deleted
es5id: 13_A11_T2
description: >
    Checking if deleting the arguments property fails and then
    returning it
flags: [noStrict]
---*/

function __func(){ 
    delete arguments;
    return arguments;
}

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (typeof __func("A","B",1,2) !== "object") {
	throw new Test262Error('#1: arguments property has attribute { DontDelete }');
}
//
//////////////////////////////////////////////////////////////////////////////
