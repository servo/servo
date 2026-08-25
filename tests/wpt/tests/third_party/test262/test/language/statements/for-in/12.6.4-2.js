// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 12.6.4-2
description: >
    The for-in Statement - the values of [[Enumerable]] attributes are
    not considered when determining if a property of a prototype
    object is shadowed by a previous object on the prototype chain
---*/

        var proto = {
            prop: "enumerableValue"
        };

        var ConstructFun = function () { };
        ConstructFun.prototype = proto;

        var child = new ConstructFun();

        Object.defineProperty(child, "prop", {
            value: "nonEnumerableValue",
            enumerable: false
        });

        var accessedProp = false;

        for (var p in child) {
            if (p === "prop") {
                accessedProp = true;
            }
        }

assert.sameValue(accessedProp, false, 'accessedProp');
