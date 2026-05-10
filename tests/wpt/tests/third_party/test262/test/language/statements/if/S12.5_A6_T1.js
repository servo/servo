// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: In the If statement expression must be enclosed in braces
es5id: 12.5_A6_T1
description: Checking if execution of "if true" fails
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if true;
//
//////////////////////////////////////////////////////////////////////////////
