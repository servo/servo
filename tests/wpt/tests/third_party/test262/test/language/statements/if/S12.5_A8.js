// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: In the "if" Statement empty expression is not allowed
es5id: 12.5_A8
description: Checking if execution of "if()" fails
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if();
//
//////////////////////////////////////////////////////////////////////////////
