// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Only three expressions and two semicolons in "for" braces are allowed.
    Appearing of for (ExpressionNoIn_opt ; Expression_opt ; Expression_opt; Expression_opt; Expression_opt;) statement leads to SyntaxError
es5id: 12.6.3_A7_T1
description: >
    Checking if execution of "for(index=0; index<10; index++;
    index--)" fails
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
for(index=0; index<10; index++; index--) ;
//
//////////////////////////////////////////////////////////////////////////////
