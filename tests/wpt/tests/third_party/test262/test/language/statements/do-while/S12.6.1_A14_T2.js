// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: FunctionExpression within a "do-while" Expression is allowed
es5id: 12.6.1_A14_T2
description: >
    Using FunctionExpression "function __func(){return 0;}()" as an
    Expression
---*/

//////////////////////////////////////////////////////////////////////////////
//CHECK#
do{
    var __reached = 1;
   break;
}while(function __func(){return 0;}());
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#2
if (__reached !== 1) {
	throw new Test262Error('#2: function expession inside of do-while expression is allowed');
}
//
//////////////////////////////////////////////////////////////////////////////
