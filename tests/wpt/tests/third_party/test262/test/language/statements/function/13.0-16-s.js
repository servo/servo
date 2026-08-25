// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Refer 13; 
    The production FunctionBody : SourceElementsopt is evaluated as follows:
es5id: 13.0-16-s
description: >
    Strict Mode - SourceElements is evaluated as strict mode code when
    a FunctionExpression is contained in strict mode code within eval
    code
flags: [noStrict]
---*/


assert.throws(SyntaxError, function() {
    eval("'use strict'; var _13_0_16_fun = function () {eval = 42;};");
    _13_0_16_fun();
});
