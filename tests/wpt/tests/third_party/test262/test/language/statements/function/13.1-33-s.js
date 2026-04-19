// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Refer 13.1; 
    It is a SyntaxError if any Identifier value occurs more than once within a FormalParameterList of a strict mode
    FunctionDeclaration or FunctionExpression.
es5id: 13.1-33-s
description: >
    Strict Mode - SyntaxError is thrown if function is created using a
    FunctionExpression that is contained in eval strict code and the
    function has three identical parameters
flags: [noStrict]
---*/


assert.throws(SyntaxError, function() {
    eval("'use strict'; var _13_1_33_fun = function (param, param, param) { };")
});
