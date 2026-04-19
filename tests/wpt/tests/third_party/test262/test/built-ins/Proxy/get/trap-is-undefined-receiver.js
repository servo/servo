// Copyright (C) 2016 Aleksey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-proxy-object-internal-methods-and-internal-slots-get-p-receiver
description: >
    Pass to target's [[Get]] correct receiver if trap is missing
info: |
    [[Get]] (P, Receiver)

    7. If trap is undefined, then
        a. Return ? target.[[Get]](P, Receiver).
features: [Proxy]
---*/

var target = {
  get attr() {
    return this;
  }
};

var p = new Proxy(target, {
  get: null
});
assert.sameValue(p.attr, p);

var pParent = Object.create(new Proxy(target, {}));
assert.sameValue(pParent.attr, pParent);
