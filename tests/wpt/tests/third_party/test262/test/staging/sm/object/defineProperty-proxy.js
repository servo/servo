/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
includes: [compareArray.js]
description: |
  pending
esid: pending
---*/
// Test details of the implementation of ToPropertyDescriptor exposed to scripts
// thanks to scriptable proxies.

// A LoggingProxy object logs certain operations performed on it.
var log = [];
function LoggingProxy(target) {
    return new Proxy(target, {
        has: function (t, id) {
            log.push("has " + id);
            return id in t;
        },
        get: function (t, id) {
            log.push("get " + id);
            return t[id];
        }
    });
}

// Tragically, we use separate code to implement Object.defineProperty on
// arrays and on proxies. So run the test three times.
var testSubjects = [
    {},
    [],
    new Proxy({}, {})
];

for (var obj of testSubjects) {
    log = [];

    // Object.defineProperty is one public method that performs a
    // ToPropertyDescriptor call.
    Object.defineProperty(obj, "x", new LoggingProxy({
        enumerable: true,
        configurable: true,
        value: 3,
        writable: true
    }));

    // It should have performed exactly these operations on the proxy, in this
    // order. See ES6 rev 24 (2014 April 27) 6.2.4.5 ToPropertyDescriptor.
    assert.compareArray(log, [
        "has enumerable", "get enumerable",
        "has configurable", "get configurable",
        "has value", "get value",
        "has writable", "get writable",
        "has get",
        "has set"
    ]);
}

