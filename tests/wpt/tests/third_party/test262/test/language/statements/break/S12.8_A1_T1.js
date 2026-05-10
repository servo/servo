// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Appearing of break without an IterationStatement leads to syntax error
es5id: 12.8_A1_T1
description: Checking if break statement with no loop fails
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
var x=1;
break;
var y=2;
//
//////////////////////////////////////////////////////////////////////////////
