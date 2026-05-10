// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 13.1-13gs
description: >
    StrictMode - SyntaxError is thrown if 'arguments' occurs as the
    Identifier of a FunctionDeclaration
negative:
  phase: parse
  type: SyntaxError
flags: [onlyStrict]
---*/

$DONOTEVALUATE();

function arguments() { };
