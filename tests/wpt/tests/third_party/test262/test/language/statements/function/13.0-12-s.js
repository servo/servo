// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Refer 13; 
    The production FunctionBody : SourceElementsopt is evaluated as follows:
es5id: 13.0-12-s
description: >
    Strict Mode - SourceElements is not evaluated as strict mode code
    when a Function constructor is contained in strict mode code and
    the function constructor body is not strict
flags: [onlyStrict]
---*/

var _13_0_12_fun = new Function(" ","eval = 42;");
_13_0_12_fun();
