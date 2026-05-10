// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: GetValue(V) mast fail
es5id: 8.7.2_A1_T2
description: Checking if execution of "1=1" fails
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

1=1;
