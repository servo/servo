// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The true is reserved word
es5id: 8.3_A2.1
description: Checking if execution of "true=1" fails
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

true = 1;
