// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 13.0-2
description: >
    13.0 - multiple names in one function declaration is not allowed,
    three function names
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

function x,y,z(){}
