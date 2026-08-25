// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Check examples for automatic semicolon insertion from the standard
es5id: 7.9.2_A1_T1
description: "{ 1 2 } 3 is not a valid sentence in the ECMAScript grammar"
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

//CHECK#1
{ 1 2 } 3
