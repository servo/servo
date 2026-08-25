// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    TryStatement: "try Block Catch" or "try Block Finally" or "try Block
    Catch Finally"
es5id: 12.14_A16_T3
description: Checking if execution of "finally" with no "try" fails
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

// CHECK#1
finally
