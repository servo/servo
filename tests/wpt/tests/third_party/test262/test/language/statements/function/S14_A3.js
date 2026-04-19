// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Global FunctionDeclaration cannot be defined within the body of another
    FunctionDeclaration
es5id: 14_A3
description: Declaring a function within the body of another function
---*/

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (typeof __func !== "function") {
	throw new Test262Error('#1: typeof __func === "function". Actual:  typeof __func ==='+ typeof __func  );
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#2
if (typeof __gunc !== "undefined") {
	throw new Test262Error('#2: typeof __gunc === "undefined". Actual:  typeof __gunc ==='+ typeof __gunc  );
}
//
//////////////////////////////////////////////////////////////////////////////

function __func(){
    function __gunc(){return true};
}
