// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: use-strict-directive
es5id: 10.1.1-25-s
description: >
    Strict Mode - Function code of Accessor PropertyAssignment
    contains Use Strict Directive which appears at the start of the
    block(getter)
flags: [noStrict]
---*/


assert.throws(ReferenceError, function() {
            var obj = {};
            Object.defineProperty(obj, "accProperty", {
                get: function () {
                    "use strict";
                    test262unresolvable = null;
                    return 11;
                }
            });
            var temp = obj.accProperty === 11;
});
