// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: use-strict-directive
es5id: 10.1.1-18-s
description: >
    Strict Mode - Function code that is part of a Accessor
    PropertyAssignment is in Strict Mode if Accessor
    PropertyAssignment is contained in use strict(setter)
flags: [noStrict]
---*/

var data = "data";

assert.throws(ReferenceError, function() {
        "use strict";

            var obj = {};
            Object.defineProperty(obj, "accProperty", {
                set: function (value) {
                    test262unresolvable = null;
                    data = value;
                }
            });

            obj.accProperty = "overrideData";
});

assert.sameValue(data, "data", 'data unchanged');
