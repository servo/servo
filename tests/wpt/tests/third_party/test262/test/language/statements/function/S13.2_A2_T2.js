// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Nested function are admitted
es5id: 13.2_A2_T2
description: Nesting level is three
---*/

var __ROBOT="C3PO";

function __FUNC(){
    function __GUNC(){
        return arguments[0];
    };
    function __HUNC(){
        return __GUNC;
    };
    return __HUNC;
};

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (__FUNC()()(__ROBOT) !== __ROBOT) {
	throw new Test262Error('#1: __FUNC()()(__ROBOT) === __ROBOT. Actual: __FUNC()()(__ROBOT) ==='+__FUNC()()(__ROBOT));
}
//
//////////////////////////////////////////////////////////////////////////////
