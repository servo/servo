// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 13.1-39-s
description: >
    StrictMode - SyntaxError is thrown if 'arguments' occurs as the
    function name of a FunctionDeclaration in strict eval code
flags: [noStrict]
---*/


assert.throws(SyntaxError, function() {
    eval("'use strict'; function arguments() { };")
});
