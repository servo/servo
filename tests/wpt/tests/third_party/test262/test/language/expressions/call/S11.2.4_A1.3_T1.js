// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Arguments : (ArgumentList : ArgumentList,, AssignmentExpression) is a bad
    syntax
es5id: 11.2.4_A1.3_T1
description: incorrect syntax
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

function f_arg() {
}

f_arg(1,,2);
