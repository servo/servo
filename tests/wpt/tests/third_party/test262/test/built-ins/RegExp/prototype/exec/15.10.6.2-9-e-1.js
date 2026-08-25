// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.10.6.2-9-e-1
description: >
    RegExp.prototype.exec - the removed step 9.e won't affected
    current algorithm
---*/

        var str = "Hello World!";
        var regObj = new RegExp("World");
        var result = false;
        result = regObj.exec(str).toString() === "World";

assert(result, 'result !== true');
