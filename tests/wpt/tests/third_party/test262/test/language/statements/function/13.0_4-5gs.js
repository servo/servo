// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 13.0_4-5gs
description: >
    Strict Mode - SourceElements is evaluated as strict mode code when
    a FunctionDeclaration is contained in strict mode code
negative:
  phase: parse
  type: SyntaxError
flags: [onlyStrict]
---*/

$DONOTEVALUATE();

function _13_0_4_5_fun() { eval = 42; };
