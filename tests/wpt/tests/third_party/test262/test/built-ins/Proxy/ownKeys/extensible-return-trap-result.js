// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-proxy-object-internal-methods-and-internal-slots-ownpropertykeys
description: >
    If target is extensible, return the non-falsy trap result if it contains all
    of target's non-configurable keys.
info: |
    [[OwnPropertyKeys]] ( )

    ...
    18. If extensibleTarget is true, return trapResult.
features: [Proxy]
---*/

var target = {};

Object.defineProperty(target, "foo", {
  configurable: false,
  enumerable: true,
  value: true
});

var p = new Proxy(target, {
  ownKeys: function() {
    return ["foo", "bar"];
  }
});

var keys = Object.getOwnPropertyNames(p);

assert.sameValue(keys[0], "foo");
assert.sameValue(keys[1], "bar");

assert.sameValue(keys.length, 2);
