// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-proxy-object-internal-methods-and-internal-slots-ownpropertykeys
description: >
    If target is extensible, return the non-falsy trap result if target doesn't
    contain any non-configurable keys.
info: |
    [[OwnPropertyKeys]] ( )

    ...
    15. If extensibleTarget is true and targetNonconfigurableKeys is empty, then
        a. Return trapResult.
features: [Proxy]
---*/

var p = new Proxy({
  attr: 42
}, {
  ownKeys: function() {
    return ["foo", "bar"];
  }
});

var keys = Object.getOwnPropertyNames(p);

assert.sameValue(keys[0], "foo");
assert.sameValue(keys[1], "bar");

assert.sameValue(keys.length, 2);
