// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 10.4.3-1-35-s
description: >
    Strict Mode - checking 'this' (Anonymous FunctionExpression
    defined within an Anonymous FunctionExpression inside strict mode)
flags: [onlyStrict]
---*/

(function () {
    assert.sameValue((function () {
        return typeof this;
    })(), "undefined");
    assert.sameValue(typeof this, "undefined", 'typeof this');
})();
