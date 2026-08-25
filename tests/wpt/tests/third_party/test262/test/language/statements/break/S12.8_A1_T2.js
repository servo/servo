// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Appearing of break without an IterationStatement leads to syntax error
es5id: 12.8_A1_T2
description: Checking if break Identifier with no loop fails
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

LABEL : x=3.14;

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
var x=1;
break LABEL;
var y=2;
//
//////////////////////////////////////////////////////////////////////////////
