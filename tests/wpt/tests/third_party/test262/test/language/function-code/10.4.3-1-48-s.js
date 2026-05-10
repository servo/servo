// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 10.4.3-1-48-s
description: >
    Strict Mode - checking 'this' (FunctionDeclaration with a strict
    directive prologue defined within a FunctionExpression)
flags: [noStrict]
---*/

var global = this;

var f1 = function () {
    function f() {
        "use strict";
        return typeof this;
    }
    return (f()==="undefined") && (this===global);
}

assert(f1(), 'f1() !== true');
