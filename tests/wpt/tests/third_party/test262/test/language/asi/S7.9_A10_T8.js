// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Check {} for automatic semicolon insertion
es5id: 7.9_A10_T8
description: Checking if execution of "{1 2} 3" fails
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

//CHECK#1
{1 2} 3
