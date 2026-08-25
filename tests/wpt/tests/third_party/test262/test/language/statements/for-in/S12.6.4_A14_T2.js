// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: FunctionExpession within a "for-in" Expression is allowed
es5id: 12.6.4_A14_T2
description: "Using \"function __func(){return {a:1};}()\" as Expession"
---*/

var x;

//////////////////////////////////////////////////////////////////////////////
//CHECK#
for(x in function __func(){return {a:1};}()){
    var __reached = x;
};
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#2
if (__reached !== "a") {
	throw new Test262Error('#2: function expession inside of for-in expression allowed');
}
//
//////////////////////////////////////////////////////////////////////////////
