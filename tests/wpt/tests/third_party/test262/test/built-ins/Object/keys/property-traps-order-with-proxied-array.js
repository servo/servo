// Copyright (C) 2018 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-object.keys
description: >
  Ensure the correct property traps are called on a proxy of an array.
info: |
  19.1.2.16 Object.keys ( O )
  ...
  2. Let nameList be ? EnumerableOwnPropertyNames(obj, "key").
  ...

  7.3.21 EnumerableOwnPropertyNames ( O, kind )
  ...
  2. Let ownKeys be ? O.[[OwnPropertyKeys]]().
  ...
  4. For each element key of ownKeys in List order, do
    a. If Type(key) is String, then
      i. Let desc be ? O.[[GetOwnProperty]](key).
      ...
features: [Proxy]
includes: [compareArray.js]
---*/

var log = [];

Object.keys(new Proxy([], new Proxy({},{
    get(t, pk, r) {
        log.push(pk);
    }
})));

assert.compareArray([
    "ownKeys",
    "getOwnPropertyDescriptor",
], log);
