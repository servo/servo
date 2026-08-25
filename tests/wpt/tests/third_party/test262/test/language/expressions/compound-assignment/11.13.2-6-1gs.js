// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 11.13.2-6-1gs
description: >
    Strict Mode - SyntaxError is throw if the identifier eval appears
    as the LeftHandSideExpression of a Compound Assignment operator(*=)
negative:
  phase: parse
  type: SyntaxError
flags: [onlyStrict]
---*/

$DONOTEVALUATE();

eval *= 20;
