// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Since arguments property has attribute { DontDelete }, only its elements
    can be deleted
es5id: 13_A11_T3
description: Deleting arguments[i] and returning result of the operation
---*/

function __func(){
    var was_del=false;
    for (var i=0; i < arguments.length; i++)
       was_del= was_del || delete arguments[i];
    return was_del;
}

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (!__func("A","B",1,2)) {
	throw new Test262Error('#1: Since arguments property has attribute { DontDelete } elements of arguments can be deleted');
}
//
//////////////////////////////////////////////////////////////////////////////
