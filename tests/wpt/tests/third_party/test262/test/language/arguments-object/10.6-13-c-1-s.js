// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 10.6-13-c-1-s
description: >
    Accessing callee property of Arguments object throws TypeError in
    strict mode
flags: [onlyStrict]
---*/


assert.throws(TypeError, function() {
    arguments.callee;
});
