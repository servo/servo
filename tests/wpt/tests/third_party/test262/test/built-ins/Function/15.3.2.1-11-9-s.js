// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.3.2.1-11-9-s
description: >
    Strict Mode - No SyntaxError is thrown if a function is created using
    the Function constructor that has three identical parameters and
    there is no explicit 'use strict' in the function constructor's
    body
---*/

var foo = new Function("baz", "baz", "baz", "return 0;");
