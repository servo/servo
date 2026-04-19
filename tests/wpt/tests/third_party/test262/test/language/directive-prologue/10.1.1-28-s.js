// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 10.1.1-28-s
description: >
    Strict Mode - Function code of Accessor PropertyAssignment
    contains Use Strict Directive which appears at the end of the
    block(setter)
flags: [noStrict]
---*/

        var obj = {};
        var data;

        Object.defineProperty(obj, "accProperty", {
            set: function (value) {
                var _10_1_1_28_s = {a:1, a:2};
                data = value;
                "use strict";
            }
        });
        obj.accProperty = "overrideData";

assert.sameValue(data, "overrideData", 'data');
