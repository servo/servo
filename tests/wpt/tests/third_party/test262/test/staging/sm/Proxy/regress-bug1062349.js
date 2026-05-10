// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [deepEqual.js]
flags:
  - noStrict
description: |
  pending
esid: pending
---*/
// Adapted from a test case contributed by André Bargull in bug 1062349.

var log = [];
var hh = {
    get(t, pk) {
        log.push("trap: " + pk);
        return t[pk];
    }
};
var h = new Proxy({
    get set() {
        log.push("called set()");
        Object.defineProperty(o, "prop", {value: 0});
        log.push("o.prop:", Object.getOwnPropertyDescriptor(o, "prop"));
    }
}, hh);
var p = new Proxy({}, h);
var o = {__proto__: p};

o.prop = 1;

var expectedDesc = {value: 0, writable: false, enumerable: false, configurable: false};
assert.deepEqual(log, [
    "trap: set",
    "called set()",
    "o.prop:",
    expectedDesc
]);
assert.deepEqual(Object.getOwnPropertyDescriptor(o, "prop"), expectedDesc);

