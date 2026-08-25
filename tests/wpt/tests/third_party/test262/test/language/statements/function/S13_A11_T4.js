// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Since arguments property has attribute { DontDelete }, only its elements
    can be deleted
es5id: 13_A11_T4
description: Deleting arguments[i] and checking the type of arguments[i]
---*/

function __func(){
    var is_undef=true;
    for (var i=0; i < arguments.length; i++)
    {
        delete arguments[i];
        is_undef= is_undef && (typeof arguments[i] === "undefined");
    };       
    return is_undef;
};

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (!__func("A","B",1,2)) {
	throw new Test262Error('#1: Since arguments property has attribute { DontDelete }, but elements of arguments can be deleted');
}
//
//////////////////////////////////////////////////////////////////////////////
