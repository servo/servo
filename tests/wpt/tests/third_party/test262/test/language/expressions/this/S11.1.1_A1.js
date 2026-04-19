// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The "this" is reserved word
es5id: 11.1.1_A1
description: Checking if execution of "this=1" fails
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

this = 1;
