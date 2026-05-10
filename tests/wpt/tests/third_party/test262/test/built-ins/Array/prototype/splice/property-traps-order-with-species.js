// Copyright (C) 2018 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.splice
description: >
  Ensure the correct property traps are called on the new array.
features: [Proxy, Symbol.species]
includes: [compareArray.js]
---*/

var log = [];

var a = [0, 1];
a.constructor = {};

a.constructor[Symbol.species] = function(len) {
    return new Proxy(new Array(len), new Proxy({}, {
        get(t, pk, r) {
            log.push(pk);
        }
    }));
};

var r = a.splice(0);

assert.compareArray([
    // Step 11.c.ii: CreateDataPropertyOrThrow(A, ! ToString(k), fromValue).
    "defineProperty",

    // Step 11.c.ii: CreateDataPropertyOrThrow(A, ! ToString(k), fromValue).
    "defineProperty",

    // Step 12: Perform ? Set(A, "length", actualDeleteCount, true).
    "set",
    "getOwnPropertyDescriptor",
    "defineProperty",
], log);
