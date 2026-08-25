// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: FunctionExpression within a "while" Expression is allowed
es5id: 12.6.2_A14_T1
description: Using "function __func(){return 0;}" as an Expression
---*/

//////////////////////////////////////////////////////////////////////////////
//CHECK#
while(function __func(){return 0;}){
   var __reached = 1;
   break;
};
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#2
if (__reached !== 1) {
	throw new Test262Error('#2: function expression inside of while expression is allowed');
}
//
//////////////////////////////////////////////////////////////////////////////
