// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Refer 13; 
    The production FunctionBody : SourceElementsopt is evaluated as follows:
es5id: 13.0-14-s
description: >
    Strict Mode - SourceElements is evaluated as strict mode code when
    the function body of a Function constructor contains a Strict
    Directive
flags: [noStrict]
---*/

assert.throws(SyntaxError, function() {
    var _13_0_14_fun = new Function(" ", "'use strict'; eval = 42; ");
    _13_0_14_fun();
});
