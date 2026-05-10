// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Only AssignmentExpression is admitted when variable is initialized
es5id: 12.2_A8_T8
description: Checking if execution of "var x in __arr" fails
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

__arr = [];

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
var x in __arr;
//
//////////////////////////////////////////////////////////////////////////////
