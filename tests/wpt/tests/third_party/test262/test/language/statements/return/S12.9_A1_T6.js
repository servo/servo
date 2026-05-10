// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Appearing of "return" without a function body leads to syntax error
es5id: 12.9_A1_T6
description: >
    Checking if execution of "return" with no function, placed into a
    loop, fails
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
do {
    var x=1;
    return;
    var y=2;
} while(0);
//
//////////////////////////////////////////////////////////////////////////////
