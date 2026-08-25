// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 10.4.3-1-42-s
description: >
    Strict Mode - checking 'this' (FunctionDeclaration defined within
    an Anonymous FunctionExpression with a strict directive prologue)
flags: [noStrict]
---*/

(function () {
    "use strict";
    function f() {
        return typeof this;
    }
    assert.sameValue(f(), "undefined", 'f()');
    assert.sameValue(typeof this, "undefined", 'typeof this');
})();
