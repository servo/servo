// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 13.2-2-s
description: >
    StrictMode - A TypeError is thrown when a strict mode code writes
    to properties named 'caller' of function instances.
flags: [onlyStrict]
---*/


assert.throws(TypeError, function() {
    var foo = function () {
    }
    foo.caller = 20;
});
