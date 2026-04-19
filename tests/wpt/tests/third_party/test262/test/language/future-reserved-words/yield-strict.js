// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-identifiers-static-semantics-early-errors
es5id: 7.6.1.2-5-s
description: >
    Strict Mode - SyntaxError is thrown when ReservedWord
    'yield' occurs in strict mode code
info: |
    BindingIdentifier : yield

    It is a Syntax Error if the code matched by this production is contained in strict mode code.
negative:
  phase: parse
  type: SyntaxError
flags: [onlyStrict]
---*/

$DONOTEVALUATE();

var yield = 1;
