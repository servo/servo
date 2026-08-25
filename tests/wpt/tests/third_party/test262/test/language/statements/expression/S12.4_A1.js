// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    An ExpressionStatement can not start with the function keyword because
    that might make it ambiguous with a FunctionDeclaration
es5id: 12.4_A1
description: Checking if execution of "function(){}()" fails
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
function(){}();
//
//////////////////////////////////////////////////////////////////////////////
