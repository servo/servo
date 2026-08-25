// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 13.1-1-2
description: >
    Duplicate identifier allowed in non-strict function expression
    parameter list
flags: [noStrict]
---*/

(function foo(a,a){});
