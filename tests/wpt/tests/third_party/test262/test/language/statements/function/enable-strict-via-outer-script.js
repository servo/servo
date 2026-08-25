// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Refer 13;
    The production FunctionBody : SourceElementsopt is evaluated as follows:
es5id: 13.0-9-s
description: >
    Strict Mode - SourceElements is evaluated as strict mode code when
    a FunctionDeclaration that is contained in strict mode code has an
    inner function
negative:
  phase: parse
  type: SyntaxError
flags: [onlyStrict]
---*/

$DONOTEVALUATE();

var _13_0_9_fun = function () {
    function _13_0_9_inner() {
        eval = 42;
    }
};
