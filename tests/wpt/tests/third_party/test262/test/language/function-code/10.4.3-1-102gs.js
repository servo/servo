// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 10.4.3-1-102gs
description: >
    Strict Mode - checking 'this' (strict anonymous function passed as
    arg to String.prototype.replace)
---*/

var x = 3;
if ( ("ab".replace("b", (function () { 
    "use strict";
    return function () {
        x = this;
        return "a";
    }
})())!=="aa") || (x!==undefined)) {
    throw "'this' had incorrect value!";
}
