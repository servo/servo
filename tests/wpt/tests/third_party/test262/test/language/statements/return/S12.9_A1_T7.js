// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Appearing of "return" without a function body leads to syntax error
es5id: 12.9_A1_T7
description: >
    Checking if execution of "return x" with no function, placed
    inside Block, fails
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
{
    var x=1;
    return x;
    var y=2;
}
//
//////////////////////////////////////////////////////////////////////////////
