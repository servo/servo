// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Refer 11.1.4; 
    The production
    ElementList : ElementList , Elisionopt AssignmentExpression
    6.Call the [[DefineOwnProperty]] internal method of array with arguments ToString(ToUint32((pad+len)) and the Property Descriptor { [[Value]]: initValue
    , [[Writable]]: true, [[Enumerable]]: true, [[Configurable]]: true}, and false.
es5id: 11.1.4_5-6-1
description: >
    Initialize array using ElementList (ElementList , Elisionopt
    AssignmentExpression) when index property (read-only) exists in
    Array.prototype (step 6)
---*/

            Object.defineProperty(Array.prototype, "1", {
                value: 100,
                writable: false,
                configurable: true
            });
            var arr = [101, 12];

assert(arr.hasOwnProperty("1"), 'arr.hasOwnProperty("1") !== true');
assert.sameValue(arr[1], 12, 'arr[1]');
