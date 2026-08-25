// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Refer 13;
    The production FunctionBody : SourceElementsopt is evaluated as follows:
es5id: 13.0-10-s
description: >
    Strict Mode - SourceElements is evaluated as strict mode code when
    the code of this FunctionBody with an inner function contains a
    Use Strict Directive
negative:
  phase: parse
  type: SyntaxError
flags: [noStrict]
---*/

$DONOTEVALUATE();

function _13_0_10_fun() {
    function _13_0_10_inner() {
        "use strict";
        eval = 42;
    }
}
