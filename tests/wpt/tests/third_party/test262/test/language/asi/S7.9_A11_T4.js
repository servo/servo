// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Check If Statement for automatic semicolon insertion
es5id: 7.9_A11_T4
description: Checking if execution of "if (false) x = 1 else x = -1" fails
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

//CHECK#1
var x = 0;
if (false) x = 1 else x = -1
