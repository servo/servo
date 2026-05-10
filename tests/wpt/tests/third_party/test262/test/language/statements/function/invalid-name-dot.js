// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 13.0-3
description: >
    13.0 - property names in function definition is not allowed, add a
    new property into object
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

function obj.tt() {}
