// Copyright (C) 2019 Aleksey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-proxy-object-internal-methods-and-internal-slots-delete-p
description: >
    Throw a TypeError exception if trap result is true, targetDesc is configurable,
    and target is not extensible.
info: |
    [[Delete]] (P)

    ...
    13. Let extensibleTarget be ? IsExtensible(target).
    14. If extensibleTarget is false, throw a TypeError exception.
    ...
features: [Proxy, Reflect, proxy-missing-checks]
---*/

var trapCalls = 0;
var p = new Proxy({prop: 1}, {
  deleteProperty: function(t, prop) {
    Object.preventExtensions(t);
    trapCalls++;
    return true;
  },
});

assert.throws(TypeError, function() {
  Reflect.deleteProperty(p, "prop");
});
assert.sameValue(trapCalls, 1);

assert(Reflect.deleteProperty(p, "nonExistent"));
assert.sameValue(trapCalls, 2);
