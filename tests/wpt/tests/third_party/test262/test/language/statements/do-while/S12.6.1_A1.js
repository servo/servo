// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    When the production "do Statement while ( Expression )" is evaluated,
    Statement is evaluated first
es5id: 12.6.1_A1
description: Evaluating various Expressions
---*/

var __in__do;

do __in__do=1; while ( false );

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (__in__do!==1) {
	throw new Test262Error('#1: the inner statement of a do-loop should be evaluated before the expression: false evaluates to false');
}
//
//////////////////////////////////////////////////////////////////////////////

do __in__do=2; while ( 0 );

//////////////////////////////////////////////////////////////////////////////
//CHECK#2
if (__in__do!==2) {
	throw new Test262Error('#2: the inner statement of a do-loop should be evaluated before the expression: 0 evaluates to false');
}
//
//////////////////////////////////////////////////////////////////////////////

do __in__do=3; while ( "" );

//////////////////////////////////////////////////////////////////////////////
//CHECK#3
if (__in__do!==3) {
	throw new Test262Error('#3: the inner statement of a do-loop should be evaluated before the expression: "" evaluates to false');
}
//
//////////////////////////////////////////////////////////////////////////////
