// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 13.1-35-s
description: >
    StrictMode - SyntaxError is thrown if 'eval' occurs as the
    function name of a FunctionDeclaration in strict eval code
flags: [noStrict]
---*/


assert.throws(SyntaxError, function() {
            eval("'use strict'; function eval() { };")
});
