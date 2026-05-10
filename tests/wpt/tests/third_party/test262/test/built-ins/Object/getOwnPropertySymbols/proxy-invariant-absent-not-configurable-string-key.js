// Copyright (C) 2019 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-object.getownpropertysymbols
description: >
  Proxy [[OwnPropertyKeys]] trap does not skip string keys when validating invariant:
  * The result List must contain the keys of all non-configurable own properties of
    the target object.
info: |
  Object.getOwnPropertySymbols ( O )

  1. Return ? GetOwnPropertyKeys(O, Symbol).

  GetOwnPropertyKeys ( O, type )

  ...
  2. Let keys be ? obj.[[OwnPropertyKeys]]().

  [[OwnPropertyKeys]] ( )

  ...
  11. Let targetKeys be ? target.[[OwnPropertyKeys]]().
  ...
  15. Let targetNonconfigurableKeys be a new empty List.
  16. For each element key of targetKeys, do
    a. Let desc be ? target.[[GetOwnProperty]](key).
    b. If desc is not undefined and desc.[[Configurable]] is false, then
      i. Append key as an element of targetNonconfigurableKeys.
  ...
  18. Let uncheckedResultKeys be a new List which is a copy of trapResult.
  19. For each key that is an element of targetNonconfigurableKeys, do
    a. If key is not an element of uncheckedResultKeys, throw a TypeError exception.
features: [Proxy]
---*/

var target = {};
Object.defineProperty(target, 'prop', {
  value: 1,
  writable: true,
  enumerable: true,
  configurable: false,
});

var proxy = new Proxy(target, {
  ownKeys: function() {
    return [];
  },
});

assert.throws(TypeError, function() {
  Object.getOwnPropertySymbols(proxy);
});
