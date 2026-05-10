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

  [[OwnPropertyKeys]] ( )

  1. Let keys be a new empty List.
  [...]
  5. For each integer i starting with 0 such that i < len, in ascending order, do
    a. Add ! ToString(i) as the last element of keys.
  [...]
  7. For each own property key P of O such that Type(P) is String and P is not an
  array index, in ascending chronological order of property creation, do
    a. Add P as the last element of keys.
  8. For each own property key P of O such that Type(P) is Symbol, in ascending
  chronological order of property creation, do
    a. Add P as the last element of keys.
  9. Return keys.
includes: [compareArray.js]
features: [Symbol, Proxy, Reflect]
---*/

var sym = Symbol();
var string = new String("str");
string[sym] = 1;

var stringTarget = new Proxy(string, {});
var stringProxy = new Proxy(stringTarget, {});

assert.compareArray(
  Reflect.ownKeys(stringProxy),
  ["0", "1", "2", "length", sym]
);
