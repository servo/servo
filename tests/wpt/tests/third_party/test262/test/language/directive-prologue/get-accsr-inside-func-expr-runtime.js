// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: use-strict-directive
es5id: 10.1.1-17-s
description: >
    Strict Mode - Function code that is part of a Accessor
    PropertyAssignment is in Strict Mode if Accessor
    PropertyAssignment is contained in use strict(getter)
flags: [noStrict]
---*/

assert.throws(ReferenceError, function() {
        "use strict";
            var obj = {};
            Object.defineProperty(obj, "accProperty", {
                get: function () {
                    test262unresolvable = null;
                    return 11;
                }
            });

            var temp = obj.accProperty === 11;
});
