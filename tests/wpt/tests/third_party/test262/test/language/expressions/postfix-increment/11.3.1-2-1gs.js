// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 11.3.1-2-1gs
description: >
    Strict Mode - SyntaxError is throw if the identifier arguments
    appear as a PostfixExpression(arguments++)
negative:
  phase: parse
  type: SyntaxError
flags: [onlyStrict]
---*/

$DONOTEVALUATE();

arguments++;
