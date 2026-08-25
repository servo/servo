// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Refer 11.1.5; 
    The production
    PropertyNameAndValueList : PropertyAssignment 
    3.Call the [[DefineOwnProperty]] internal method of obj with arguments propId.name, propId.descriptor, and false.
es5id: 11.1.5_3-3-1
description: >
    Object initialization using PropertyNameAndValueList
    (PropertyAssignment) when property (read-only) exists in
    Object.prototype (step 3)
---*/

            Object.defineProperty(Object.prototype, "prop", {
                value: 100,
                writable: false,
                configurable: true
            });
            var obj = { prop: 12 };

assert(obj.hasOwnProperty("prop"), 'obj.hasOwnProperty("prop") !== true');
assert.sameValue(obj.prop, 12, 'obj.prop');
