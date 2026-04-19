// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: use-strict-directive
es5id: 10.1.1-26-s
description: >
    Strict Mode - Function code of Accessor PropertyAssignment
    contains Use Strict Directive which appears at the start of the
    block(setter)
flags: [noStrict]
---*/

var data = "data";

assert.throws(ReferenceError, function() {
            var obj = {};
            Object.defineProperty(obj, "accProperty", {
                set: function (value) {
                    "use strict";
                    test262unresolvable = null;
                    data = value;
                }
            });

            obj.accProperty = "overrideData";
});

assert.sameValue(data, "data", 'data unchanged');
