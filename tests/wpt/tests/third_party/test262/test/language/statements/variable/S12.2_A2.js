// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Variables are defined with global scope (that is, they are created as
    members of the global object, as described in 10.1.3) using property
    attributes { DontDelete}
es5id: 12.2_A2
description: >
    Checking if deleting global variables that have the attributes
    {DontDelete} fails
flags: [noStrict]
---*/

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (delete(__variable)) {
	throw new Test262Error('#1: delete(__variable)===false');
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#2
if (delete(this["__variable"])) {
	throw new Test262Error('#2: delete(this["__variable"])===false');
}
//
//////////////////////////////////////////////////////////////////////////////


var __variable;
var __variable = "defined";

//////////////////////////////////////////////////////////////////////////////
//CHECK#3
if (delete(__variable) | delete(this["__variable"])) {
	throw new Test262Error('#3: (delete(__variable) | delete(this["__variable"]))===false' );
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#4
if ((__variable !== "defined")|(this["__variable"] !=="defined")) {
	throw new Test262Error('#4: __variable === "defined" and this["__variable"] ==="defined"');
}
//
//////////////////////////////////////////////////////////////////////////////
