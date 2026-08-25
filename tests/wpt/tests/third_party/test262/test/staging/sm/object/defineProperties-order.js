// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
// Based on testcases provided by André Bargull

let log = [];
let logger = new Proxy({}, {
    get(target, key) {
        log.push(key);
    }
});

Object.create(null, new Proxy({a: {value: 0}, b: {value: 1}}, logger));
assert.sameValue(log.join(), "ownKeys,getOwnPropertyDescriptor,get,getOwnPropertyDescriptor,get");

log = [];
Object.defineProperties({}, new Proxy({a: {value: 0}, b: {value: 1}}, logger));
assert.sameValue(log.join(), "ownKeys,getOwnPropertyDescriptor,get,getOwnPropertyDescriptor,get");

