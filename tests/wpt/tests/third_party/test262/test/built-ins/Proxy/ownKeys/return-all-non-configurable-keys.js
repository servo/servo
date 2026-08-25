// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-proxy-object-internal-methods-and-internal-slots-ownpropertykeys
description: >
    The result List must contain the keys of all non-configurable own properties
    of the target object.
info: |
    [[OwnPropertyKeys]] ( )

    ...
    17. For each key that is an element of targetNonconfigurableKeys, do
        a. If key is not an element of uncheckedResultKeys, throw a TypeError
        exception.

features: [Proxy]
---*/

var target = {
  foo: 1
};

Object.defineProperty(target, "attr", {
  configurable: false,
  enumerable: true,
  value: true
});

var p = new Proxy(target, {
  ownKeys: function() {
    return ["foo"];
  }
});

assert.throws(TypeError, function() {
  Object.keys(p);
});
