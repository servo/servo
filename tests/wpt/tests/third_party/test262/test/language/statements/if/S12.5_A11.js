// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: "{} within the \"if\" expression is not allowed"
es5id: 12.5_A11
description: Checking if execution of "if({1})" fails
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

//////////////////////////////////////////////////////////////////////////////
//CHECK#
if({1})
  {
    ;
  }else
  {
    ;
  }
//
//////////////////////////////////////////////////////////////////////////////
