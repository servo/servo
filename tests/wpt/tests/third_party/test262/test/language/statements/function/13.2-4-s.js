// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 13.2-4-s
description: >
    StrictMode - A TypeError is thrown when a code in strict mode
    tries to write to 'arguments' of function instances.
flags: [onlyStrict]
---*/


assert.throws(TypeError, function() {
    var foo = function () {
    }
    foo.arguments = 20;
});
