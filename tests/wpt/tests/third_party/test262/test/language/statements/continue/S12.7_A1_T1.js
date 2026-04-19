// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Appearing of continue without an IterationStatement leads to syntax error
es5id: 12.7_A1_T1
description: >
    Checking if execution of single "continue" without any
    IterationStatement fails
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
var x=1;
continue;
var y=2;
//
//////////////////////////////////////////////////////////////////////////////
