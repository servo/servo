// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.3.2.1-11-8-s
description: >
    Strict Mode - SyntaxError is not thrown if a function is created
    using a Function constructor that has two identical parameters,
    which are separated by a unique parameter name and there is no
    explicit 'use strict' in the function constructor's body
flags: [onlyStrict]
---*/

var foo = new Function("baz", "qux", "baz", "return 0;");
