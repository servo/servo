// Copyright (C) 2019 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-object.getownpropertysymbols
description: >
  Proxy [[OwnPropertyKeys]] trap does not skip string keys when validating invariant:
  * If the target object is not extensible, then the result List must contain all the keys of
    the own properties of the target object and no other values.
info: |
  Object.getOwnPropertySymbols ( O )

  1. Return ? GetOwnPropertyKeys(O, Symbol).

  GetOwnPropertyKeys ( O, type )

  ...
  2. Let keys be ? obj.[[OwnPropertyKeys]]().

  [[OwnPropertyKeys]] ( )

  ...
  11. Let targetKeys be ? target.[[OwnPropertyKeys]]().
  16. For each element key of targetKeys, do
    a. Let desc be ? target.[[GetOwnProperty]](key).
    b. If desc is not undefined and desc.[[Configurable]] is false, then
      ...
    c. Else,
      i. Append key as an element of targetConfigurableKeys.
  ...
  18. Let uncheckedResultKeys be a new List which is a copy of trapResult.
  ...
  21. For each key that is an element of targetConfigurableKeys, do
    a. If key is not an element of uncheckedResultKeys, throw a TypeError exception.
features: [Proxy]
---*/

var target = {prop: 2};
var proxy = new Proxy(target, {
  ownKeys: function() {
    return [];
  },
});

Object.preventExtensions(target);

assert.throws(TypeError, function() {
  Object.getOwnPropertySymbols(proxy);
});
