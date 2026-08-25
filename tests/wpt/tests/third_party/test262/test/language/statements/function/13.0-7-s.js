// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Refer 13; 
    The production FunctionBody : SourceElementsopt is evaluated as follows:
es5id: 13.0-7-s
description: >
    Strict Mode - SourceElements is evaluated as strict mode code when
    the code of this FunctionDeclaration is contained in non-strict
    mode but the call to eval is a direct call in strict mode code
flags: [noStrict]
---*/

assert.throws(SyntaxError, function() {
    eval("'use strict'; function _13_0_7_fun() {eval = 42;};");
    _13_0_7_fun();
});
