// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Refer 11.1.4; 
    The production
    ElementList : Elisionopt AssignmentExpression
    5.Call the [[DefineOwnProperty]] internal method of array with arguments ToString(firstIndex), the Property Descriptor { [[Value]]: initValue, [[Writable]]: true
    , [[Enumerable]]: true, [[Configurable]]: true}, and false.
es5id: 11.1.4_4-5-1
description: >
    Initialize array using ElementList (Elisionopt
    AssignmentExpression) when index property (read-only) exists in
    Array.prototype (step 5)
---*/

            Object.defineProperty(Array.prototype, "0", {
                value: 100,
                writable: false,
                configurable: true
            });
            var arr = [101];

assert(arr.hasOwnProperty("0"), 'arr.hasOwnProperty("0") !== true');
assert.sameValue(arr[0], 101, 'arr[0]');
