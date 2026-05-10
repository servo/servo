// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-object.seal
description: >
  If Proxy "ownKeys" trap is missing, keys are sorted by type in ascending
  chronological order.
info: |
  SetIntegrityLevel ( O, level )

  [...]
  5. Let keys be ? O.[[OwnPropertyKeys]]().
  6. If level is sealed, then
    a. For each element k of keys, do
      i. Perform ? DefinePropertyOrThrow(O, k, PropertyDescriptor { [[Configurable]]: false }).

  [[OwnPropertyKeys]] ( )

  [...]
  6. If trap is undefined, then
    a. Return ? target.[[OwnPropertyKeys]]().

  OrdinaryOwnPropertyKeys ( O )

  [...]
  3. For each own property key P of O such that Type(P) is String and P is
  not an array index, in ascending chronological order of property creation, do
    a. Add P as the last element of keys.
  4. For each own property key P of O such that Type(P) is Symbol,
  in ascending chronological order of property creation, do
    a. Add P as the last element of keys.
  5. Return keys.
features: [Proxy, Symbol, Reflect]
includes: [compareArray.js]
---*/

var target = {};
var sym = Symbol();
target[sym] = 1;
target.foo = 2;
target[0] = 3;

var definePropertyKeys = [];
var proxy = new Proxy(target, {
  defineProperty: function(target, key, descriptor) {
    definePropertyKeys.push(key);
    return Reflect.defineProperty(target, key, descriptor);
  },
});

Object.seal(proxy);
assert.compareArray(definePropertyKeys, ["0", "foo", sym]);
