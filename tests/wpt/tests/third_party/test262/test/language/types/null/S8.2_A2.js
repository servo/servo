// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The null is resrved word
es5id: 8.2_A2
description: Checking if execution of "var null" fails
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

var null;
