// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Since arguments property has attribute { DontDelete }, only its elements
    can be deleted
es5id: 13_A11_T1
description: Returning result of "delete arguments"
flags: [noStrict]
---*/

function __func(){ return delete arguments;}

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (__func("A","B",1,2)) {
	throw new Test262Error('#1: arguments property has attribute { DontDelete }');
}
//
//////////////////////////////////////////////////////////////////////////////
