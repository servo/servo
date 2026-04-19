// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 13.1-2-2
description: >
    eval allowed as formal parameter name of a non-strict function
    expression
flags: [noStrict]
---*/

(function foo(eval){});
