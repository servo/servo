// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Expression in "do-while" IterationStatement is bracketed with braces
es5id: 12.6.1_A6_T1
description: Checking if execution of "do{} while 1" fails
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
do break; while 1;
//
//////////////////////////////////////////////////////////////////////////////
