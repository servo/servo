// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 12.2.1-1gs
description: >
    Strict Mode - SyntaxError is thrown if a VariableDeclaration
    occurs within strict code and its Identifier is eval
negative:
  phase: parse
  type: SyntaxError
flags: [onlyStrict]
---*/

$DONOTEVALUATE();

for (var eval in arrObj) { }
