// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: When using property attributes, {DontEnum} is not used
es5id: 12.2_A9
description: >
    Enumerating property attributes of "this" and then searching for
    the declared variable
---*/

var enumed;

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
for (var __prop in this){
    if (__prop === "__declared__var")
        enumed=true;
}
if (!(enumed)) {
	throw new Test262Error('#1: When using property attributes, {DontEnum} not used');
}
//
//////////////////////////////////////////////////////////////////////////////

var __declared__var;
