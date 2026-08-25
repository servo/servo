// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-proxy-object-internal-methods-and-internal-slots-ownpropertykeys
description: >
  If "ownKeys" trap is null or undefined, [[OwnPropertyKeys]] call is
  properly forwarded to [[ProxyTarget]] (which is also a Proxy object).
info: |
  [[OwnPropertyKeys]] ( )

  [...]
  4. Let target be O.[[ProxyTarget]].
  5. Let trap be ? GetMethod(handler, "ownKeys").
  6. If trap is undefined, then
    a. Return ? target.[[OwnPropertyKeys]]().

  OrdinaryOwnPropertyKeys ( O )

  1. Let keys be a new empty List.
  2. For each own property key P of O that is an array index,
  in ascending numeric index order, do
    a. Add P as the last element of keys.
  3. For each own property key P of O that is a String but is not an
  array index, in ascending chronological order of property creation, do
    a. Add P as the last element of keys.
  [...]
  5. Return keys.
includes: [compareArray.js]
features: [Proxy]
---*/

var plainObject = {
  foo: 1,
  "0": 2,
  get bar() {},
  "1": 4,
};

var plainObjectTarget = new Proxy(plainObject, {});
var plainObjectProxy = new Proxy(plainObjectTarget, {
  ownKeys: null,
});

assert.compareArray(
  Object.keys(plainObjectProxy),
  ["0", "1", "foo", "bar"]
);
