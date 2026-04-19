// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    "in"-expression is not allowed as a ExpressionNoIn in "for
    (ExpressionNoIn; FirstExpression; SecondExpression) Statement"
    IterationStatement
es5id: 12.6.3_A4_T2
description: Checking if execution of "for (1 in arr;1;){}" fails
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

arr = [1,2,3,4,5];

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
for(1 in arr;1;) {
    break;
}
//
//////////////////////////////////////////////////////////////////////////////
