// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 11.4.5-2-2gs
description: >
    Strict Mode - SyntaxError is throw if the UnaryExpression operated
    upon by a Prefix Decrement operator(--arguments)
negative:
  phase: parse
  type: SyntaxError
flags: [onlyStrict]
---*/

$DONOTEVALUATE();

--arguments;
