// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    TryStatement: "try Block Catch" or "try Block Finally" or "try Block
    Catch Finally"
es5id: 12.14_A16_T13
description: >
    Catch: "catch (Identifier ) Block". Checking if execution of "22"
    passes at the place of Identifier of "catch"
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

// CHECK#1
try
{
}
catch("22")
{
}
