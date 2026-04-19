// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 13.1-41-s
description: >
    StrictMode - SyntaxError is thrown if 'arguments' occurs as the
    Identifier of a FunctionExpression in strict eval code
flags: [noStrict]
---*/

var _13_1_41_s = {};

assert.throws(SyntaxError, function() {
    eval("'use strict'; _13_1_41_s.x = function arguments() {};");
});
