// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 10.1.1-29-s
description: >
    Strict Mode - The built-in Function constructor is contained in
    use strict code
flags: [noStrict]
---*/

function testcase() {
        "use strict";
        var funObj = new Function("a", "eval('public = 1;');");
        funObj();
    }
testcase();
